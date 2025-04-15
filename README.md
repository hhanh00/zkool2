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
- [x] Account Manager
    - [x] CRUD
    - [x] reorder
    - [x] hide accounts
- [x] Synchronization
    - [x] transparent
    - [x] shielded
    - [x] rewind
    - [x] memo
    - [x] reorg detection
    - [x] continuous
    - [x] retry
    - [x] scan past transparent addresses
    - [x] reset
    - [x] height progress observers per account
- [x] Receive
    - [x] default address generation
    - [x] diversified address generation
    - [x] additional transparent addresses
- [x] Send
    - [x] multiple payment editor
        - [x] address, amount
        - [x] memo
    - [x] builder
    - [x] pczt
    - [x] broadcast
    - [x] generate change transparent addresses
    - [x] expose src pools and receipient pays fee
- [x] History
    - [x] tx list
    - [x] memo
    - [x] split views
- [x] QR
    - [x] scanner
    - [x] show
- [x] Log viewer
    - [x] integrate tracing framework
    - [x] make log viewer page
    - [x] add logging messages
- [ ] Market price
- [x] MultiSend
- [ ] payment URI generation
- [x] Export
    - [x] accounts
    - [x] tx history
    - [x] encryption
- [x] Import
    - [x] accounts
    - [x] tx history
    - [x] encryption
- [ ] Database
    - [x] switch to new database
    - [ ] encrypt database with AES
- [ ] Settings
    - [x] database name
    - [ ] lwd url
    - [ ] fiat currency
    - [ ] min confs
    - [ ] get tx details
    - [ ] protect open/send
- [ ] Tx details page
- [ ] App Icons
- [ ] Bugs
    - [x] No native certs
    - [x] duplicate accounts sync - identical nullifiers: only one account is
    detected
    - [ ] heights should update in account list after sync
    - [ ] key scope column

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
