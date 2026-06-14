use anyhow::Result;
#[cfg(feature = "flutter")]
use flutter_rust_bridge::frb;

use crate::api::coin::Coin;
use crate::contacts;

#[cfg_attr(feature = "flutter", frb(dart_metadata = ("freezed")))]
pub struct Contact {
    pub id: u32,
    pub name: String,
    pub addresses: Vec<String>,
    pub notes: String,
}

#[cfg_attr(feature = "flutter", frb(dart_metadata = ("freezed")))]
pub struct ContactMatch {
    pub contact: Contact,
    pub matched_address: String,
}

#[cfg_attr(feature = "flutter", frb)]
pub async fn list_contacts(c: &Coin) -> Result<Vec<Contact>> {
    let mut connection = c.get_connection().await?;
    let result = contacts::list_contacts(&mut connection).await?;
    Ok(result.into_iter().map(|c| c.into()).collect())
}

#[cfg_attr(feature = "flutter", frb)]
pub async fn create_contact(
    name: &str,
    addresses: Vec<String>,
    notes: &str,
    c: &Coin,
) -> Result<Contact> {
    let mut connection = c.get_connection().await?;
    let network = c.network();
    let result = contacts::create_contact(&mut connection, name, &addresses, notes, &network).await?;
    Ok(result.into())
}

#[cfg_attr(feature = "flutter", frb)]
pub async fn update_contact(
    id: u32,
    name: Option<String>,
    addresses: Option<Vec<String>>,
    notes: Option<String>,
    c: &Coin,
) -> Result<()> {
    let mut connection = c.get_connection().await?;
    let network = c.network();
    contacts::update_contact(
        &mut connection,
        id,
        name.as_deref(),
        addresses.as_deref(),
        notes.as_deref(),
        &network,
    )
    .await
}

#[cfg_attr(feature = "flutter", frb)]
pub async fn delete_contacts(ids: Vec<u32>, c: &Coin) -> Result<()> {
    let mut connection = c.get_connection().await?;
    contacts::delete_contacts(&mut connection, &ids).await
}

/// Find contacts whose stored addresses match the given address.
///
/// The input address can be either a unified address (which will be expanded
/// to its constituent receivers) or a single-pool receiver address.
/// Returns matching contacts with the original address that produced the match.
#[cfg_attr(feature = "flutter", frb)]
pub async fn find_contacts_for_address(address: &str, c: &Coin) -> Result<Vec<ContactMatch>> {
    let mut connection = c.get_connection().await?;
    let network = c.network();

    // Expand the input address to (receiver, pool) pairs
    let expanded = contacts::expand_address_to_receivers_with_pool(address, &network)?;

    let mut results: Vec<ContactMatch> = Vec::new();
    let mut seen_ids = std::collections::HashSet::new();

    for (receiver, pool) in &expanded {
        let matches =
            contacts::find_contacts_for_address(&mut connection, receiver, *pool).await?;
        for (contact, matched_address) in matches {
            if seen_ids.insert((contact.id, matched_address.clone())) {
                results.push(ContactMatch {
                    contact: contact.into(),
                    matched_address,
                });
            }
        }
    }

    Ok(results)
}

#[cfg_attr(feature = "flutter", frb)]
pub async fn export_contacts_vcard(c: &Coin) -> Result<String> {
    let mut connection = c.get_connection().await?;
    contacts::export_contacts_vcard(&mut connection).await
}

#[cfg_attr(feature = "flutter", frb)]
pub async fn import_contacts_vcard(vcard_data: &str, c: &Coin) -> Result<Vec<Contact>> {
    let mut connection = c.get_connection().await?;
    let network = c.network();
    let result = contacts::import_contacts_vcard(&mut connection, vcard_data, &network).await?;
    Ok(result.into_iter().map(|c| c.into()).collect())
}

impl From<contacts::Contact> for Contact {
    fn from(c: contacts::Contact) -> Self {
        Contact {
            id: c.id,
            name: c.name,
            addresses: c.addresses,
            notes: c.notes,
        }
    }
}
