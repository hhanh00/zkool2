# Read me

## Main Page / Account Manager
- Create and restore new accounts: "+" button
    - Account can be restored from seed phrase + passphrase + account index,
    sapling secret key, transparent extended key, viewing key, etc.
    - Import from encrypted file
    - Create FROST account with Distributed Key Generation
- Edit, Reset, Delete account: select icon, then edit/delete button
    (multi select supported)
- Account Edit Page
    - Change name, icon, birth height
    - Enable/Disable from default sync
    - Set hidden flag
    - Export account as encrypted file
    - Show Keys, Show Seed
- Drag & Drop to reorder
- Tap on account to go to Account Page
- Tap on Height to update and show current Market Price
- Tap on Market Price to show Market Chart
- Tap on Mempool to go to Mempool Page
- Tap on Settings to go to the Settings Page
- Tap on Sync to synchronize all the *enabled* accounts
- Tap on View Hidden Account to toggle account visibility

## Account Page
- Log Button: Open App Logs (mostly for troubleshooting)
- Sync Button: Sync this account only
- Receive Button: Go to Address Page
- Send Button: Go to Send Page
- Transaction/Memo/Note Tab
- Tap Transaction to go to Transaction Details
- Memo are searchable
- Tap notes to toggle exclusion from payments

## Receive Page
- Search Button to Scan for related Transparent Addresses
- Tap New Diversified to get a new set of addresses (including Transparent)

## Send Page
- Use buttons to Shield/Unshield all to/from Orchard
- Or, Enter or Scan Address
- Enter Amount or Tap Max amount
- Or Load an previous Transaction
- If FROST account, tap to FROST button start a multiparty signature
- Tap Add to add as a recipient to current transaction
- Tap Next to finish listing recipients and continue
- Select which Pools to use
- Select whether the fees should be deducted from the first recipient
- Select whether change amount less than 0.00005 ZEC should be given to recipient or discarded
    (dust amount are not accepted by some full nodes)
- Then View transaction and Send/Cancel/Save

## Settings Page
- Choose another Database
    - Database file contains *all* application data including
        - account keys
        - synchronization state
    - Recommend using the built-in encryption
    - *Changing db requires app restart*
    - If new db, app asks for a password. Leave blank for no password
    - To change/delete password, enter the name of the database and tap
        edit/delete button. *The current database cannot be edited/deleted*.
        Switch to the default database if needed
    - Default db is zkool. It cannot be deleted and encrypted since it is the fallback
- Server URL
    - Enter URL like: https://zec.rocks
    - Full Nodes are supported but "Light node" should be toggled off
- Actions per Sync is the size of the batches of sync. Increase to process
more blocks at once but it increases memory requirement
- AutoSync Interval is the minimum number of blocks before Auto Syncing activates
Set to 0 to disable autosync. Tap on Cancel to stop the current sync.
Alternatively, close the app. Synchronization will resume from the last checkpoint

