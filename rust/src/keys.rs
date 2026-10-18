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

// ── Viewing keys of an account in a pool ─────────────────────────────────

/// An account's Sapling viewing keys: its full viewing key, or, for an
/// incoming-only account imported from an incoming viewing key, the external
/// incoming key alone. The incoming key finds payments received and recovers
/// their diversifier index; it has no internal scope and no outgoing key.
pub enum SaplingViewingKeys {
    Full(sapling_crypto::zip32::DiversifiableFullViewingKey),
    Incoming(sapling_crypto::zip32::IncomingViewingKey),
}

impl SaplingViewingKeys {
    /// From the stored columns: the full key when there is one, else the
    /// incoming key, else nothing.
    pub fn from_stored(xvk: Option<Vec<u8>>, ivk: Option<Vec<u8>>) -> Option<Self> {
        match (xvk, ivk) {
            (Some(xvk), _) => Some(SaplingViewingKeys::Full(
                sapling_crypto::zip32::DiversifiableFullViewingKey::from_bytes(
                    &xvk.try_into().unwrap(),
                )
                .unwrap(),
            )),
            (None, Some(ivk)) => Some(SaplingViewingKeys::Incoming(
                sapling_crypto::zip32::IncomingViewingKey::from_bytes(&ivk.try_into().unwrap())
                    .unwrap(),
            )),
            (None, None) => None,
        }
    }

    /// The incoming keys to trial-decrypt with, by scope: both scopes of a
    /// full key, the external scope of an incoming key.
    pub fn prepared_ivks(&self) -> Vec<(u8, sapling_crypto::keys::PreparedIncomingViewingKey)> {
        match self {
            SaplingViewingKeys::Full(dfvk) => [zip32::Scope::External, zip32::Scope::Internal]
                .into_iter()
                .map(|scope| {
                    (
                        scope_to_u8(scope),
                        sapling_crypto::keys::PreparedIncomingViewingKey::new(&dfvk.to_ivk(scope)),
                    )
                })
                .collect(),
            SaplingViewingKeys::Incoming(ivk) => vec![(0, ivk.prepare())],
        }
    }

    pub fn external_prepared_ivk(&self) -> sapling_crypto::keys::PreparedIncomingViewingKey {
        match self {
            SaplingViewingKeys::Full(dfvk) => sapling_crypto::keys::PreparedIncomingViewingKey::new(
                &dfvk.to_ivk(zip32::Scope::External),
            ),
            SaplingViewingKeys::Incoming(ivk) => ivk.prepare(),
        }
    }

    /// The external outgoing viewing key, which recovers what the account
    /// sent; an incoming key has none.
    pub fn external_ovk(&self) -> Option<sapling_crypto::keys::OutgoingViewingKey> {
        match self {
            SaplingViewingKeys::Full(dfvk) => Some(dfvk.fvk().ovk),
            SaplingViewingKeys::Incoming(_) => None,
        }
    }

    /// The diversifier index of a received note from its raw diversifier,
    /// when the key produced that address; an incoming key only for the
    /// external scope.
    pub fn diversifier_index(&self, scope: u8, diversifier: &[u8]) -> Option<i64> {
        match self {
            SaplingViewingKeys::Full(dfvk) => {
                crate::db::resolve_sapling_diversifier_index(dfvk, scope, diversifier)
            }
            SaplingViewingKeys::Incoming(_) if scope != 0 => None,
            SaplingViewingKeys::Incoming(ivk) => {
                let d = sapling_crypto::keys::Diversifier(diversifier.try_into().ok()?);
                let address = ivk.address(d)?;
                ivk.decrypt_diversifier(&address)
                    .and_then(|di| di.try_into().ok())
                    .map(|d: u64| d as i64)
            }
        }
    }
}

/// As [`SaplingViewingKeys`], for Orchard; the same keys serve Ironwood.
pub enum OrchardViewingKeys {
    Full(orchard::keys::FullViewingKey),
    Incoming(orchard::keys::IncomingViewingKey),
}

impl OrchardViewingKeys {
    pub fn from_stored(xvk: Option<Vec<u8>>, ivk: Option<Vec<u8>>) -> Option<Self> {
        match (xvk, ivk) {
            (Some(xvk), _) => Some(OrchardViewingKeys::Full(
                orchard::keys::FullViewingKey::from_bytes(&xvk.try_into().unwrap()).unwrap(),
            )),
            (None, Some(ivk)) => Some(OrchardViewingKeys::Incoming(
                orchard::keys::IncomingViewingKey::from_bytes(&ivk.try_into().unwrap()).unwrap(),
            )),
            (None, None) => None,
        }
    }

    pub fn prepared_ivks(&self) -> Vec<(u8, orchard::keys::PreparedIncomingViewingKey)> {
        match self {
            OrchardViewingKeys::Full(fvk) => {
                [orchard::keys::Scope::External, orchard::keys::Scope::Internal]
                    .into_iter()
                    .map(|scope| {
                        (
                            orchard_scope_to_u8(scope),
                            orchard::keys::PreparedIncomingViewingKey::new(&fvk.to_ivk(scope)),
                        )
                    })
                    .collect()
            }
            OrchardViewingKeys::Incoming(ivk) => {
                vec![(0, orchard::keys::PreparedIncomingViewingKey::new(ivk))]
            }
        }
    }

    pub fn external_prepared_ivk(&self) -> orchard::keys::PreparedIncomingViewingKey {
        match self {
            OrchardViewingKeys::Full(fvk) => orchard::keys::PreparedIncomingViewingKey::new(
                &fvk.to_ivk(orchard::keys::Scope::External),
            ),
            OrchardViewingKeys::Incoming(ivk) => orchard::keys::PreparedIncomingViewingKey::new(ivk),
        }
    }

    pub fn external_ovk(&self) -> Option<orchard::keys::OutgoingViewingKey> {
        match self {
            OrchardViewingKeys::Full(fvk) => Some(fvk.to_ovk(orchard::keys::Scope::External)),
            OrchardViewingKeys::Incoming(_) => None,
        }
    }

    pub fn diversifier_index(&self, scope: u8, diversifier: &[u8]) -> Option<i64> {
        match self {
            OrchardViewingKeys::Full(fvk) => {
                crate::db::resolve_orchard_diversifier_index(fvk, scope, diversifier)
            }
            OrchardViewingKeys::Incoming(_) if scope != 0 => None,
            OrchardViewingKeys::Incoming(ivk) => {
                let d = orchard::keys::Diversifier::from_bytes(diversifier.try_into().ok()?);
                let address = ivk.address(d);
                ivk.diversifier_index(&address)
                    .and_then(|di| di.try_into().ok())
                    .map(|d: u64| d as i64)
            }
        }
    }
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

/// An account's shielded viewing keys, full or incoming-only per pool: what
/// rebuilds the address a note was received on.
pub struct AccountViewingKeys {
    pub sapling: Option<SaplingViewingKeys>,
    pub orchard: Option<OrchardViewingKeys>,
}

impl AccountViewingKeys {
    pub fn from_ufvk(ufvk: &zcash_keys::keys::UnifiedFullViewingKey) -> Self {
        AccountViewingKeys {
            sapling: ufvk.sapling().cloned().map(SaplingViewingKeys::Full),
            orchard: ufvk.orchard().cloned().map(OrchardViewingKeys::Full),
        }
    }

    /// Rebuilds the receiving address of a note from its pool, scope and raw
    /// diversifier. Pool 1 is Sapling; pools 2 and 3 (Orchard, Ironwood)
    /// share the Orchard key. Transparent notes carry no diversifier and
    /// yield `None`. An incoming key has external addresses only.
    pub fn note_address(
        &self,
        network: &crate::api::coin::Network,
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
                let keys = self
                    .sapling
                    .as_ref()
                    .ok_or_else(|| anyhow::anyhow!("account has no sapling key"))?;
                let d = sapling_crypto::keys::Diversifier(diversifier);
                let (address, diversifier_index) = match keys {
                    SaplingViewingKeys::Full(sfvk) => {
                        let address = sfvk.diversified_address_for_scope(scope, d).ok_or_else(
                            || anyhow::anyhow!("Sapling diversified address derivation failed"),
                        )?;
                        let di = sfvk.decrypt_diversifier(&address).and_then(|d| d.0.try_into().ok());
                        (address, di)
                    }
                    SaplingViewingKeys::Incoming(_) if scope != 0 => {
                        anyhow::bail!("an incoming-only account has no internal addresses")
                    }
                    SaplingViewingKeys::Incoming(ivk) => {
                        let address = ivk.address(d).ok_or_else(|| {
                            anyhow::anyhow!("Sapling diversified address derivation failed")
                        })?;
                        let di = ivk.decrypt_diversifier(&address).and_then(|d| d.try_into().ok());
                        (address, di)
                    }
                };
                Ok(Some(NoteAddress {
                    address: address.encode(network),
                    diversifier_index,
                }))
            }
            2 | 3 => {
                let keys = self
                    .orchard
                    .as_ref()
                    .ok_or_else(|| anyhow::anyhow!("account has no orchard key"))?;
                let d = orchard::keys::Diversifier::from_bytes(diversifier);
                let (ivk, address) = match keys {
                    OrchardViewingKeys::Full(ofvk) => orchard_ivk_and_address(ofvk, scope, d),
                    OrchardViewingKeys::Incoming(_) if scope != 0 => {
                        anyhow::bail!("an incoming-only account has no internal addresses")
                    }
                    OrchardViewingKeys::Incoming(ivk) => (ivk.clone(), ivk.address(d)),
                };
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
}

/// [`AccountViewingKeys::note_address`] for a full viewing key.
pub fn note_address(
    network: &crate::api::coin::Network,
    ufvk: &zcash_keys::keys::UnifiedFullViewingKey,
    pool: u8,
    scope: u8,
    diversifier: &[u8],
) -> anyhow::Result<Option<NoteAddress>> {
    AccountViewingKeys::from_ufvk(ufvk).note_address(network, pool, scope, diversifier)
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
