CREATE TABLE IF NOT EXISTS write_cache_transactions_input(
    txn_number                                  INTEGER PRIMARY KEY,

    is_faild                                    BOOLEAN,
    user_                                       TEXT,
    txn                                         BLOB
);

CREATE TABLE IF NOT EXISTS write_cache_transactions_result(
    txn_number                                  INTEGER PRIMARY KEY,

    user_                                       TEXT,
    txn                                         BLOB
);
