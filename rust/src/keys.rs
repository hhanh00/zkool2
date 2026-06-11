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
) -> sapling_crypto::zip32::ExtendedSpendingKey {
    if scope.is_external() {
        ssk.clone()
    } else {
        ssk.derive_internal()
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
        (
            vk.fvk().vk.ivk(),
            vk.to_nk(zip32::Scope::External),
        )
    } else {
        (
            vk.to_internal_fvk().vk.ivk(),
            vk.to_nk(zip32::Scope::Internal),
        )
    }
}
