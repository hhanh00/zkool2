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
    - [x] multi edit for enabled & hidden
- [x] Account Manager
    - [x] CRUD
    - [x] reorder
    - [x] hide accounts
    - [x] show balance
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
- [x] MultiSend
- [x] Export
    - [x] accounts
    - [x] tx history
    - [x] encryption
- [x] Import
    - [x] accounts
    - [x] tx history
    - [x] encryption
- [x] Database
    - [x] switch to new database
    - [x] encrypt database with AES
- [x] Transparent sweep
- [ ] Settings
    - [x] database name
    - [x] lwd url
    - [ ] fiat currency
    - [ ] min confs
    - [x] get tx details
    - [ ] protect open/send
- [x] Market price
- [x] Tx details page
- [x] Payment URI generation
- [x] App Icons
- [x] Mempool monitoring
- [x] offline signing, PCZT
- [x] multisig accounts
    - [x] DKG
    - [x] FROST

## User Stories

In order of priority,

### Memos
- [x] Sync Tx Details
- [x] Send
- [x] Display
- [x] Search

### Rotate Transparent Addresses
- [x] Auto New change address
- [x] Manual New receive address
- [x] Sweep past addresses

### Account Import/Export
- [x] Single/Multi account export
- [x] Add/Import to current database
