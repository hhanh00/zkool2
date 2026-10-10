-- Wallet-local 1Click history. Base-unit amounts remain TEXT because destination
-- assets can exceed SQLite's signed 64-bit integer range. Provider status is
-- deliberately unrestricted so new 1Click states can be stored unchanged.
CREATE TABLE IF NOT EXISTS swaps (
    id_swap INTEGER PRIMARY KEY,
    account INTEGER NOT NULL REFERENCES accounts(id_account) ON DELETE CASCADE,
    origin_asset TEXT NOT NULL,
    destination_asset TEXT NOT NULL,
    swap_type TEXT NOT NULL CHECK (swap_type IN ('EXACT_INPUT', 'EXACT_OUTPUT')),
    amount TEXT NOT NULL CHECK (length(amount) > 0 AND amount NOT GLOB '*[^0-9]*' AND amount GLOB '*[1-9]*'),
    slippage_tolerance INTEGER NOT NULL CHECK (slippage_tolerance BETWEEN 0 AND 10000),
    recipient TEXT NOT NULL,
    refund_to TEXT NOT NULL,
    deadline TEXT NOT NULL,
    amount_in TEXT NOT NULL CHECK (length(amount_in) > 0 AND amount_in NOT GLOB '*[^0-9]*' AND amount_in GLOB '*[1-9]*'),
    amount_out TEXT NOT NULL CHECK (length(amount_out) > 0 AND amount_out NOT GLOB '*[^0-9]*' AND amount_out GLOB '*[1-9]*'),
    min_amount_in TEXT,
    min_amount_out TEXT,
    deposit_address TEXT NOT NULL UNIQUE CHECK (length(trim(deposit_address)) > 0),
    deposit_memo TEXT,
    -- Display-order hex hash, independent of transactions: a broadcast deposit
    -- must survive wallet rescans and may not yet appear in transaction history.
    deposit_tx_hash TEXT CHECK (deposit_tx_hash IS NULL OR
        (length(deposit_tx_hash) = 64 AND deposit_tx_hash NOT GLOB '*[^0-9a-fA-F]*')),
    deposit_submitted_at INTEGER,
    quote_response TEXT NOT NULL,
    status TEXT,
    status_response TEXT,
    last_checked_at INTEGER,
    -- Local tracking completion is separate from the provider's status string.
    completed_at INTEGER,
    created_at INTEGER NOT NULL DEFAULT (unixepoch()),
    updated_at INTEGER NOT NULL DEFAULT (unixepoch())
);

CREATE INDEX IF NOT EXISTS swaps_account_history ON swaps(account, created_at DESC, id_swap DESC);
CREATE INDEX IF NOT EXISTS swaps_pending ON swaps(account, last_checked_at) WHERE completed_at IS NULL;
