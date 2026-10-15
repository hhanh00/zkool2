//! Key and scope utilities — unified abstractions for scope-based branching.
//!
//! The central idea: numeric scope (0 = external, non-zero = internal) is
//! converted into strongly-typed scopes and used to derive addresses / select
//! keys via traits implemented on the key types themselves.  Call sites that
//! previously repeated `if scope == 0 { … } else { … }` now call a single
//! trait method or free function.

use orchard;
use sapling_crypto;
use zcash_transparent::keys::TransparentKeyScope;
use zip32;

// ── ScopeExt ──────────────────────────────────────────────────────────────

/// Convert a numeric scope (`u8` or `u32`) into the typed scope for the
/// relevant key type, or simply ask whether it represents the external pool.
pub trait ScopeExt {
    /// `true` when the scope is `0` (external).
    fn is_external(self) -> bool;

    /// `true` when the scope is non-zero (internal).
    fn is_internal(self) -> bool
    where
        Self: Sized,
    {
        !self.is_external()
    }

    /// Convert to [`orchard::keys::Scope`].
    fn orchard_scope(self) -> orchard::keys::Scope;

    /// Convert to [`zip32::Scope`] (Sapling).
    fn sapling_scope(self) -> zip32::Scope;

    /// Convert to [`TransparentKeyScope`].
    fn transparent_scope(self) -> TransparentKeyScope;
}

impl ScopeExt for u8 {
    fn is_external(self) -> bool {
        self == 0
    }

    fn orchard_scope(self) -> orchard::keys::Scope {
        if self == 0 {
            orchard::keys::Scope::External
        } else {
            orchard::keys::Scope::Internal
        }
    }

    fn sapling_scope(self) -> zip32::Scope {
        if self == 0 {
            zip32::Scope::External
        } else {
            zip32::Scope::Internal
        }
    }

    fn transparent_scope(self) -> TransparentKeyScope {
        match self {
            0 => TransparentKeyScope::EXTERNAL,
            1 => TransparentKeyScope::INTERNAL,
            _ => unreachable!(),
        }
    }
}

impl ScopeExt for u32 {
    fn is_external(self) -> bool {
        self == 0
    }

    fn orchard_scope(self) -> orchard::keys::Scope {
        (self as u8).orchard_scope()
    }

    fn sapling_scope(self) -> zip32::Scope {
        (self as u8).sapling_scope()
    }

    fn transparent_scope(self) -> TransparentKeyScope {
        (self as u8).transparent_scope()
    }
}

// ── Scope → u8 (inverse) ─────────────────────────────────────────────────

/// Convert a typed Sapling scope back to a numeric `u8`.
pub fn scope_to_u8(scope: zip32::Scope) -> u8 {
    match scope {
        zip32::Scope::External => 0,
        zip32::Scope::Internal => 1,
    }
}

/// Convert a typed Orchard scope back to a numeric `u8`.
pub fn orchard_scope_to_u8(scope: orchard::keys::Scope) -> u8 {
    match scope {
        orchard::keys::Scope::External => 0,
        orchard::keys::Scope::Internal => 1,
    }
}

// ── Sapling address derivation from DFVK ─────────────────────────────────

/// Derive a Sapling payment address from a DFVK + optional internal IVK,
/// dispatching on scope automatically.
///
/// - Scope 0 (external): uses [`DiversifiableFullViewingKey::address`].
/// - Scope ≠ 0 (internal): uses the internal IVK's `address_at`.
pub trait SaplingAddressDerivation {
    fn sapling_address_at(
        &self,
        scope: impl ScopeExt,
        d: u64,
        internal_ivk: Option<&sapling_crypto::zip32::IncomingViewingKey>,
    ) -> Option<sapling_crypto::PaymentAddress>;

    fn has_sapling_address(
        &self,
        scope: impl ScopeExt,
        d: u64,
        internal_ivk: Option<&sapling_crypto::zip32::IncomingViewingKey>,
    ) -> bool {
        self.sapling_address_at(scope, d, internal_ivk).is_some()
    }
}

impl SaplingAddressDerivation for sapling_crypto::zip32::DiversifiableFullViewingKey {
    fn sapling_address_at(
        &self,
        scope: impl ScopeExt,
        d: u64,
        internal_ivk: Option<&sapling_crypto::zip32::IncomingViewingKey>,
    ) -> Option<sapling_crypto::PaymentAddress> {
        if scope.is_external() {
            self.address(d.into())
        } else {
            internal_ivk.and_then(|ivk| ivk.address_at(d))
        }
    }
}

// ── Sapling diversified-address resolver ──────────────────────────────────

/// Resolve a diversifier into a payment address, using the correct
/// `diversified_address` / `diversified_change_address` for the scope.
pub trait SaplingDiversifiedAddress {
    fn diversified_address_for_scope(
        &self,
        scope: impl ScopeExt,
        d: sapling_crypto::keys::Diversifier,
    ) -> Option<sapling_crypto::PaymentAddress>;
}

impl SaplingDiversifiedAddress for sapling_crypto::zip32::DiversifiableFullViewingKey {
    fn diversified_address_for_scope(
        &self,
        scope: impl ScopeExt,
        d: sapling_crypto::keys::Diversifier,
    ) -> Option<sapling_crypto::PaymentAddress> {
        if scope.is_external() {
            self.diversified_address(d)
        } else {
            self.diversified_change_address(d)
        }
    }
}

// ── Orchard address derivation from FVK ───────────────────────────────────

/// Derive an Orchard address from a full viewing key, dispatching on scope.
pub trait OrchardAddressDerivation {
    fn orchard_address_at(&self, scope: impl ScopeExt, dindex: u64) -> orchard::Address;
}

impl OrchardAddressDerivation for orchard::keys::FullViewingKey {
    fn orchard_address_at(&self, scope: impl ScopeExt, dindex: u64) -> orchard::Address {
        self.address_at(dindex, scope.orchard_scope())
    }
}

/// Convenience: derive both the IVK and address for an Orchard FVK.
pub fn orchard_ivk_and_address(
    ofvk: &orchard::keys::FullViewingKey,
    scope: impl ScopeExt,
    d: orchard::keys::Diversifier,
) -> (orchard::keys::IncomingViewingKey, orchard::Address) {
    let s = scope.orchard_scope();
    (ofvk.to_ivk(s), ofvk.address(d, s))
}

// ── Note address recovery ────────────────────────────────────────────────

/// The address a shielded note was received on, rebuilt from the note's raw
/// diversifier, and that address's diversifier index when the viewing key
/// can recover it.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct NoteAddress {
    pub address: String,
    pub diversifier_index: Option<u64>,
}

/// Rebuilds the receiving address of a note from its pool, scope and raw
/// diversifier. Pool 1 is Sapling; pools 2 and 3 (Orchard, Ironwood) share
/// the Orchard key. Transparent notes carry no diversifier and yield `None`.
pub fn note_address(
    network: &crate::api::coin::Network,
    ufvk: &zcash_keys::keys::UnifiedFullViewingKey,
    pool: u8,
    scope: u8,
    diversifier: &[u8],
) -> anyhow::Result<Option<NoteAddress>> {
    use zcash_keys::encoding::AddressCodec as _;

    let diversifier: [u8; 11] = diversifier.try_into().map_err(|_| {
        anyhow::anyhow!("diversifier wrong length: {} bytes", diversifier.len())
    })?;
    match pool {
        1 => {
            let sfvk = ufvk
                .sapling()
                .ok_or_else(|| anyhow::anyhow!("UFVK missing sapling key"))?;
            let address = sfvk
                .diversified_address_for_scope(scope, sapling_crypto::keys::Diversifier(diversifier))
                .ok_or_else(|| anyhow::anyhow!("Sapling diversified address derivation failed"))?;
            let diversifier_index = sfvk
                .decrypt_diversifier(&address)
                .and_then(|d| d.0.try_into().ok());
            Ok(Some(NoteAddress {
                address: address.encode(network),
                diversifier_index,
            }))
        }
        2 | 3 => {
            let ofvk = ufvk
                .orchard()
                .ok_or_else(|| anyhow::anyhow!("UFVK missing orchard key"))?;
            let (ivk, address) =
                orchard_ivk_and_address(ofvk, scope, orchard::keys::Diversifier::from_bytes(diversifier));
            let diversifier_index = ivk
                .diversifier_index(&address)
                .and_then(|d| d.try_into().ok());
            let ua = zcash_keys::address::UnifiedAddress::from_receivers(
                Some(address),
                None,
                None,
                None,
                None,
            )
            .ok_or_else(|| anyhow::anyhow!("UnifiedAddress::from_receivers returned None"))?;
            Ok(Some(NoteAddress {
                address: ua.encode(network),
                diversifier_index,
            }))
        }
        _ => Ok(None),
    }
}

#[cfg(test)]
mod note_address_tests {
    use super::{note_address, NoteAddress};
    use crate::api::coin::Network;
    use zcash_keys::{
        address::UnifiedAddress,
        encoding::AddressCodec as _,
        keys::{UnifiedAddressRequest, UnifiedFullViewingKey, UnifiedSpendingKey},
    };

    fn ufvk() -> UnifiedFullViewingKey {
        UnifiedSpendingKey::from_seed(&Network::Main, &[7u8; 32], zip32::AccountId::ZERO)
            .expect("usk")
            .to_unified_full_viewing_key()
    }

    #[test]
    fn transparent_notes_have_no_address() {
        assert_eq!(note_address(&Network::Main, &ufvk(), 0, 0, &[0u8; 11]).unwrap(), None);
    }

    #[test]
    fn rejects_a_diversifier_of_the_wrong_length() {
        assert!(note_address(&Network::Main, &ufvk(), 2, 0, &[0u8; 10]).is_err());
    }

    #[test]
    fn recovers_the_external_sapling_and_orchard_addresses() {
        let ufvk = ufvk();
        let (ua, di) = ufvk
            .default_address(UnifiedAddressRequest::AllAvailableKeys)
            .expect("default address");
        let di = u64::try_from(di).expect("small index");

        let sapling = ua.sapling().expect("sapling receiver");
        let got = note_address(&Network::Main, &ufvk, 1, 0, &sapling.diversifier().0)
            .unwrap()
            .expect("sapling address");
        assert_eq!(
            got,
            NoteAddress {
                address: sapling.encode(&Network::Main),
                diversifier_index: Some(di),
            }
        );

        let orchard = *ua.orchard().expect("orchard receiver");
        let expected = UnifiedAddress::from_receivers(Some(orchard), None, None, None, None)
            .unwrap()
            .encode(&Network::Main);
        for pool in [2u8, 3u8] {
            let got = note_address(&Network::Main, &ufvk, pool, 0, orchard.diversifier().as_array())
                .unwrap()
                .expect("orchard address");
            assert_eq!(
                got,
                NoteAddress {
                    address: expected.clone(),
                    diversifier_index: Some(di),
                }
            );
        }
    }

    #[test]
    fn recovers_internal_addresses_by_scope() {
        let ufvk = ufvk();

        let ofvk = ufvk.orchard().expect("orchard key");
        let internal = ofvk.address_at(5u64, orchard::keys::Scope::Internal);
        let got = note_address(&Network::Main, &ufvk, 2, 1, internal.diversifier().as_array())
            .unwrap()
            .expect("orchard change address");
        assert_eq!(got.diversifier_index, Some(5));
        assert_eq!(
            got.address,
            UnifiedAddress::from_receivers(Some(internal), None, None, None, None)
                .unwrap()
                .encode(&Network::Main)
        );
        // The same diversifier under the external scope is a different address.
        let external = note_address(&Network::Main, &ufvk, 2, 0, internal.diversifier().as_array())
            .unwrap()
            .unwrap();
        assert_ne!(external.address, got.address);

        let sfvk = ufvk.sapling().expect("sapling key");
        let (_, change) = sfvk.change_address();
        let got = note_address(&Network::Main, &ufvk, 1, 1, &change.diversifier().0)
            .unwrap()
            .expect("sapling change address");
        assert_eq!(got.address, change.encode(&Network::Main));
    }
}

// ── Sapling FVK selection ────────────────────────────────────────────────

/// Return the external or internal [`sapling_crypto::keys::FullViewingKey`]
/// for a given scope.
pub trait SaplingFullViewingKey {
    fn to_fvk(&self, scope: impl ScopeExt) -> sapling_crypto::keys::FullViewingKey;
}

impl SaplingFullViewingKey for sapling_crypto::zip32::DiversifiableFullViewingKey {
    fn to_fvk(&self, scope: impl ScopeExt) -> sapling_crypto::keys::FullViewingKey {
        if scope.is_external() {
            self.fvk().clone()
        } else {
            self.to_internal_fvk()
        }
    }
}

/// Existing public helper — kept as a thin wrapper for backward compatibility.
pub fn sapling_dfvk_to_fvk(
    scope: u32,
    dfvk: &sapling_crypto::zip32::DiversifiableFullViewingKey,
) -> sapling_crypto::keys::FullViewingKey {
    dfvk.to_fvk(scope)
}

// ── Sapling signing-key selection ────────────────────────────────────────

/// Select the proof generation key for the given scope.
pub fn sapling_pgk_for_scope(
    scope: impl ScopeExt,
    pgk: sapling_crypto::ProofGenerationKey,
    internal_pgk: sapling_crypto::ProofGenerationKey,
) -> sapling_crypto::ProofGenerationKey {
    if scope.is_external() {
        pgk
    } else {
        internal_pgk
    }
}

/// Derive the spending key for the given scope.
pub fn sapling_ssk_for_scope(
    scope: impl ScopeExt,
    ssk: &sapling_crypto::zip32::ExtendedSpendingKey,
) -> anyhow::Result<sapling_crypto::zip32::ExtendedSpendingKey> {
    if scope.is_external() {
        Ok(ssk.clone())
    } else {
        ssk.derive_internal()
            .ok_or_else(|| anyhow::anyhow!("invalid internal Sapling spending key"))
    }
}

/// Return `(ivk, nk)` for the given scope, used during shielded sync.
pub fn sapling_ivk_nk_for_scope(
    scope: impl ScopeExt,
    vk: &sapling_crypto::zip32::DiversifiableFullViewingKey,
) -> (
    sapling_crypto::keys::SaplingIvk,
    sapling_crypto::keys::NullifierDerivingKey,
) {
    if scope.is_external() {
        (vk.fvk().vk.ivk(), vk.to_nk(zip32::Scope::External))
    } else {
        (
            vk.to_internal_fvk().vk.ivk(),
            vk.to_nk(zip32::Scope::Internal),
        )
    }
}
