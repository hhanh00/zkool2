# zkool

A wallet for zcash

## Roadmap

- [x] Account create/restore
    - [x] by seed & index
    - [x] by sapling secret key/viewing key
    - [x] by unified viewing key
    - [x] by xpub/xpriv key
    - [x] by bip38 extended priv/pub key
    - [x] and of course random
- [x] Account properties editor
    - [x] name
    - [x] birth height
    - [x] icon
    - [x] enabled
    - [x] hidden
- [ ] Account Manager
    - [x] CRUD
    - [x] reorder
    - [ ] hide accounts
- [x] Synchronization
    - [x] transparent
    - [x] shielded
    - [x] rewind
    - [x] memo
    - [ ] reorg detection
    - [ ] continuous
    - [ ] retry
    - [x] scan past transparent addresses
- [ ] Receive
    - [x] default address generation
    - [x] diversified address generation
    - [x] additional transparent addresses
- [ ] Send
    - [x] multiple payment editor
        - [x] address, amount
        - [x] memo
    - [x] builder
    - [x] pczt
    - [x] broadcast
    - [x] generate change transparent addresses
    - [ ] expose src pools and receipient pays fee
- [ ] History
    - [x] tx list
    - [x] memo
- [ ] QR
    - [ ] scanner
    - [ ] show
- [ ] Import/Export
    - [ ] accounts
    - [ ] tx history
    - [ ] encryption
- [ ] Database
    - [ ] switch to new database
    - [ ] encrypt database with AES

## User Stories

In order of priority,

### Memos
- [x] Sync Tx Details
- [x] Send
- [x] Display
- [ ] Search

### Rotate Transparent Addresses
- [ ] Auto New change address
- [ ] Manual New receive address
- [ ] Sweep past addresses

### Account Import/Export
- [ ] Single/Multi account export
- [ ] Add/Import to current database

### Contacts
TBD
