# Contacts

## Data

- Username: String
- Addresses: List<String>
- Notes: Multiline string

## Features

- Contact Editor
    - Import/Export as vcard, put address as note
    - Link with native device contacts
- Output address should be matched against contacts (in rust)
    - Output address is a receiver
    - Contact address is a unified/native address
    - May not match because a unified address != receiver
    - Need to expand a contact address to a list of receivers

