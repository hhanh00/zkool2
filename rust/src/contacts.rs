use anyhow::{anyhow, Result};
use sqlx::{Row, SqliteConnection};
use sqlx::sqlite::SqliteRow;
use tracing::info;
use vcard4::property::{AnyProperty, TextProperty};

/// Check if a string is a syntactically valid Zcash address (any network).
fn is_valid_zcash_address(s: &str) -> bool {
    zcash_address::ZcashAddress::try_from_encoded(s).is_ok()
}
use vcard4::Vcard;
use zcash_address::unified::{Address as UnifiedAddress, Container, Encoding, Receiver};
use zcash_address::ZcashAddress;
use zcash_keys::encoding::AddressCodec;
use zcash_protocol::consensus::Parameters;
use zcash_protocol::{PoolType, ShieldedProtocol};
use zcash_transparent::address::TransparentAddress;

use crate::api::coin::Network;

#[derive(Debug, Clone)]
pub struct Contact {
    pub id: u32,
    pub name: String,
    pub addresses: Vec<String>,
    pub notes: String,
}

/// Expand an address to a list of (receiver, pool) pairs.
/// Pool: 0 = transparent, 1 = sapling, 2 = orchard.
///
/// If the address is a unified address, it is decomposed into individual
/// receivers (one per pool), each assigned the appropriate pool id.
/// If it's a single-pool address, it is returned as-is with its pool.
pub fn expand_address_to_receivers_with_pool(
    addr: &str,
    network: &Network,
) -> Result<Vec<(String, u8)>> {
    // Try to parse as a unified address first
    if let Ok((net, ua)) = UnifiedAddress::decode(addr) {
        if net != network.network_type() {
            anyhow::bail!("Invalid network for address");
        }
        let mut results = Vec::new();
        for item in ua.items() {
            match item {
                Receiver::P2pkh(pkh) => {
                    let taddr = TransparentAddress::PublicKeyHash(pkh);
                    results.push((taddr.encode(&network), 0u8));
                }
                Receiver::P2sh(sh) => {
                    let taddr = TransparentAddress::ScriptHash(sh);
                    results.push((taddr.encode(&network), 0u8));
                }
                Receiver::Sapling(s) => {
                    let saddr = sapling_crypto::PaymentAddress::from_bytes(&s).unwrap();
                    results.push((saddr.encode(&network), 1u8));
                }
                Receiver::Orchard(o) => {
                    let oaddr = orchard::Address::from_raw_address_bytes(&o)
                        .into_option()
                        .unwrap();
                    let oaddr_ua = zcash_keys::address::UnifiedAddress::from_receivers(
                        Some(oaddr),
                        None,
                        None,
                    )
                    .unwrap();
                    results.push((oaddr_ua.encode(&network), 2u8));
                }
                _ => {}
            }
        }
        if results.is_empty() {
            anyhow::bail!("Address has no recognizable receivers");
        }
        return Ok(results);
    }

    // Fallback: single-pool address (transparent, sapling)
    let zaddr = ZcashAddress::try_from_encoded(addr).map_err(|e| anyhow!("Invalid address: {e}"))?;
    let pool = if zaddr.can_receive_as(PoolType::Transparent) {
        0u8
    } else if zaddr.can_receive_as(PoolType::Shielded(ShieldedProtocol::Sapling)) {
        1u8
    } else if zaddr.can_receive_as(PoolType::Shielded(ShieldedProtocol::Orchard)) {
        2u8
    } else {
        anyhow::bail!("Unrecognized address pool");
    };
    Ok(vec![(addr.to_string(), pool)])
}

// ---------------------------------------------------------------------------
// CRUD
// ---------------------------------------------------------------------------

pub async fn list_contacts(connection: &mut SqliteConnection) -> Result<Vec<Contact>> {
    // Fetch all contact_addresses rows, grouped by contact
    let rows = sqlx::query(
        "SELECT c.id_contact, c.name, c.notes, ca.address, ca.ordinal
         FROM contacts c
         LEFT JOIN contact_addresses ca ON c.id_contact = ca.contact_id
         ORDER BY c.name, ca.ordinal",
    )
    .map(|r: SqliteRow| {
        let id: u32 = r.get(0);
        let name: String = r.get(1);
        let notes: String = r.get(2);
        let address: Option<String> = r.get(3);
        (id, name, notes, address)
    })
    .fetch_all(&mut *connection)
    .await?;

    // Group rows by contact id, collecting DISTINCT addresses
    let mut contacts: Vec<Contact> = Vec::new();
    let mut current: Option<Contact> = None;
    let mut seen_ids = std::collections::HashSet::new();

    for (id, name, notes, addr_opt) in rows {
        if !seen_ids.contains(&id) {
            if let Some(c) = current.take() {
                contacts.push(c);
            }
            seen_ids.insert(id);
            current = Some(Contact {
                id,
                name,
                notes,
                addresses: Vec::new(),
            });
        }
        if let Some(ref mut c) = current {
            if let Some(addr) = addr_opt {
                // Only add if not already present (DISTINCT per address)
                if !c.addresses.contains(&addr) {
                    c.addresses.push(addr);
                }
            }
        }
    }
    if let Some(c) = current.take() {
        contacts.push(c);
    }

    Ok(contacts)
}

pub async fn create_contact(
    connection: &mut SqliteConnection,
    name: &str,
    addresses: &[String],
    notes: &str,
    network: &Network,
) -> Result<Contact> {
    sqlx::query("INSERT INTO contacts(name, notes) VALUES (?1, ?2)")
        .bind(name)
        .bind(notes)
        .execute(&mut *connection)
        .await?;

    let id: u32 =
        sqlx::query_scalar("SELECT id_contact FROM contacts WHERE name = ?1")
            .bind(name)
            .fetch_one(&mut *connection)
            .await?;

    // Expand each address and store (receiver, pool) rows
    for (ordinal, addr) in addresses.iter().enumerate() {
        let expanded = expand_address_to_receivers_with_pool(addr, network)?;
        for (receiver, pool) in expanded {
            sqlx::query(
                "INSERT INTO contact_addresses(contact_id, address, receiver, pool, ordinal)
                 VALUES (?1, ?2, ?3, ?4, ?5)",
            )
            .bind(id)
            .bind(addr)
            .bind(&receiver)
            .bind(pool)
            .bind(ordinal as u32)
            .execute(&mut *connection)
            .await?;
        }
    }

    Ok(Contact {
        id,
        name: name.to_string(),
        addresses: addresses.to_vec(),
        notes: notes.to_string(),
    })
}

pub async fn update_contact(
    connection: &mut SqliteConnection,
    id: u32,
    name: Option<&str>,
    addresses: Option<&[String]>,
    notes: Option<&str>,
    network: &Network,
) -> Result<()> {
    if let Some(name) = name {
        sqlx::query("UPDATE contacts SET name = ?2 WHERE id_contact = ?1")
            .bind(id)
            .bind(name)
            .execute(&mut *connection)
            .await?;
    }
    if let Some(notes) = notes {
        sqlx::query("UPDATE contacts SET notes = ?2 WHERE id_contact = ?1")
            .bind(id)
            .bind(notes)
            .execute(&mut *connection)
            .await?;
    }
    if let Some(addresses) = addresses {
        // Delete old addresses and re-insert with expansion
        sqlx::query("DELETE FROM contact_addresses WHERE contact_id = ?1")
            .bind(id)
            .execute(&mut *connection)
            .await?;

        for (ordinal, addr) in addresses.iter().enumerate() {
            let expanded = expand_address_to_receivers_with_pool(addr, network)?;
            for (receiver, pool) in expanded {
                sqlx::query(
                    "INSERT INTO contact_addresses(contact_id, address, receiver, pool, ordinal)
                     VALUES (?1, ?2, ?3, ?4, ?5)",
                )
                .bind(id)
                .bind(addr)
                .bind(&receiver)
                .bind(pool)
                .bind(ordinal as u32)
                .execute(&mut *connection)
                .await?;
            }
        }
    }

    Ok(())
}

pub async fn delete_contacts(connection: &mut SqliteConnection, ids: &[u32]) -> Result<()> {
    for id in ids {
        sqlx::query("DELETE FROM contacts WHERE id_contact = ?1")
            .bind(id)
            .execute(&mut *connection)
            .await?;
    }
    Ok(())
}

// ---------------------------------------------------------------------------
// Address matching (SQL JOIN using pre-expanded receivers)
// ---------------------------------------------------------------------------

/// Find contacts whose stored addresses match the given receiver address.
/// Uses the pre-expanded `contact_addresses.receiver` column for fast JOIN lookup.
///
/// Returns pairs of (Contact, original_address_that_matched).
pub async fn find_contacts_for_address(
    connection: &mut SqliteConnection,
    address: &str,
    pool: u8,
) -> Result<Vec<(Contact, String)>> {
    let rows = sqlx::query(
        "SELECT DISTINCT c.id_contact, c.name, c.notes, ca.address
         FROM contacts c
         JOIN contact_addresses ca ON c.id_contact = ca.contact_id
         WHERE ca.receiver = ?1 AND ca.pool = ?2",
    )
    .bind(address)
    .bind(pool)
    .map(|r: SqliteRow| {
        let id: u32 = r.get(0);
        let name: String = r.get(1);
        let notes: String = r.get(2);
        let matched_address: String = r.get(3);
        (id, name, notes, matched_address)
    })
    .fetch_all(&mut *connection)
    .await?;

    let mut results = Vec::new();
    for (id, name, notes, matched_address) in rows {
        // Fetch all addresses for this contact
        let addresses: Vec<String> = sqlx::query_scalar(
            "SELECT DISTINCT address FROM contact_addresses WHERE contact_id = ?1 ORDER BY ordinal",
        )
        .bind(id)
        .fetch_all(&mut *connection)
        .await?;

        results.push((
            Contact {
                id,
                name,
                addresses,
                notes,
            },
            matched_address,
        ));
    }
    Ok(results)
}

// ---------------------------------------------------------------------------
// vCard import/export
// ---------------------------------------------------------------------------

/// Export all contacts as a vCard 4.0 string.
pub async fn export_contacts_vcard(connection: &mut SqliteConnection) -> Result<String> {
    let contacts = list_contacts(connection).await?;
    let mut output = String::new();

    for contact in &contacts {
        let mut vcard = Vcard::new(contact.name.clone());
        // Build note with addresses included
        let mut note_parts: Vec<String> = Vec::new();
        if !contact.notes.is_empty() {
            note_parts.push(contact.notes.clone());
        }
        if !contact.addresses.is_empty() {
            note_parts.push(format!(
                "Zcash addresses:\n{}",
                contact.addresses.iter().map(|a| format!("zcash:{a}")).collect::<Vec<_>>().join("\n")
            ));
        }
        let note = note_parts.join("\n\n");
        if !note.is_empty() {
            vcard.note = vec![TextProperty {
                group: None,
                value: note,
                parameters: None,
            }];
        }
        output.push_str(&vcard.to_string());
        output.push('\n');
    }

    Ok(output)
}

/// Import contacts from a vCard 4.0 string.
/// Parses the vCard data and creates contacts in the database.
/// Returns the list of imported contacts.
pub async fn import_contacts_vcard(
    connection: &mut SqliteConnection,
    vcard_data: &str,
    network: &Network,
) -> Result<Vec<Contact>> {
    let mut imported = Vec::new();

    // Split on "END:VCARD" to handle multiple vCards
    for block in vcard_data.split("END:VCARD") {
        let block = block.trim();
        if block.is_empty() {
            continue;
        }
        // Re-attach END:VCARD for the parser
        let full = format!("{block}\nEND:VCARD");
        let vcard: Vcard = match full.as_str().try_into() {
            Ok(v) => v,
            Err(e) => {
                info!("Skipping unparseable vCard: {e:?}");
                continue;
            }
        };

        // Extract formatted name (required)
        let name = vcard
            .formatted_name
            .first()
            .map(|f| f.value.clone())
            .unwrap_or_default();
        if name.is_empty() {
            continue;
        }

        // Extract note text
        let full_note = vcard
            .note
            .first()
            .map(|n| n.value.clone())
            .unwrap_or_default();

        // Parse addresses from NOTE by scanning each line for valid Zcash addresses.
        // Lines starting with "zcash:" are payment URIs; bare addresses also accepted.
        // Lines that don't parse as valid addresses are kept as user notes.
        let mut notes = String::new();
        let mut addresses: Vec<String> = Vec::new();
        for line in full_note.lines() {
            let trimmed = line.trim();
            if trimmed.is_empty() {
                continue;
            }
            // Try "zcash:<addr>" payment URI format first
            if let Some(addr) = trimmed.strip_prefix("zcash:") {
                if is_valid_zcash_address(addr) {
                    addresses.push(addr.to_string());
                    continue;
                }
            }
            // Try bare address
            if is_valid_zcash_address(trimmed) {
                addresses.push(trimmed.to_string());
                continue;
            }
            // Not an address — keep as note
            if !notes.is_empty() {
                notes.push('\n');
            }
            notes.push_str(line);
        }

        // Backward compat: also check X-ZCASH-ADDRESS extensions
        let x_addresses: Vec<String> = vcard
            .extensions
            .iter()
            .filter(|e| e.name == "X-ZCASH-ADDRESS")
            .filter_map(|e| {
                if let AnyProperty::Text(ref t) = e.value {
                    Some(t.clone())
                } else {
                    None
                }
            })
            .collect();
        for a in x_addresses {
            if !addresses.contains(&a) {
                addresses.push(a);
            }
        }

        if addresses.is_empty() {
            continue;
        }

        // Check for duplicate name
        let existing: Option<u32> = sqlx::query_scalar(
            "SELECT id_contact FROM contacts WHERE name = ?1",
        )
        .bind(&name)
        .fetch_optional(&mut *connection)
        .await?;

        if let Some(id) = existing {
            // Update existing contact
            info!("Updating existing contact '{name}' from vCard import");
            // Merge addresses: keep existing that aren't in the vcard, add new ones
            let existing_addresses: Vec<String> = sqlx::query_scalar(
                "SELECT DISTINCT address FROM contact_addresses WHERE contact_id = ?1 ORDER BY ordinal",
            )
            .bind(id)
            .fetch_all(&mut *connection)
            .await?;

            let mut merged = existing_addresses.clone();
            for addr in &addresses {
                if !merged.contains(addr) {
                    merged.push(addr.clone());
                }
            }
            update_contact(
                connection,
                id,
                Some(&name),
                Some(&merged),
                Some(&notes),
                network,
            )
            .await?;

            imported.push(Contact {
                id,
                name,
                addresses: addresses.clone(),
                notes,
            });
        } else {
            // Create new contact
            let contact = create_contact(connection, &name, &addresses, &notes, network).await?;
            imported.push(contact);
        }
    }

    Ok(imported)
}
