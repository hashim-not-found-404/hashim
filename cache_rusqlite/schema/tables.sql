CREATE TABLE IF NOT EXISTS user(
    rowid                                       TEXT PRIMARY KEY,

    name                                        TEXT,
    id                                          TEXT,
    jwt                                         TEXT
);

CREATE TABLE IF NOT EXISTS person_out_side_the_system(
    rowid                                       TEXT PRIMARY KEY,

    name                                        TEXT
);

CREATE TABLE IF NOT EXISTS company(
    rowid                                       TEXT PRIMARY KEY,

    name                                        TEXT,
    currency                                    TEXT
);

CREATE TABLE IF NOT EXISTS company_branch(
    rowid                                       TEXT PRIMARY KEY,

    company_belong                              TEXT REFERENCES company(rowid) ON DELETE CASCADE,
    name                                        TEXT,
    location_latitude                           DECIMAL(9,6)  CHECK (location_latitude BETWEEN -90 AND 90),
    location_longitude                          DECIMAL(10,6)  CHECK (location_longitude BETWEEN -180 AND 180),
    currency                                    TEXT
);

CREATE TABLE IF NOT EXISTS photo(
    rowid                                       TEXT PRIMARY KEY,

    photo                                       BLOB
);

CREATE TABLE IF NOT EXISTS product(
    rowid                                       TEXT PRIMARY KEY,

    name                                        TEXT,
    primary_photo                               TEXT REFERENCES photo(rowid) ON DELETE CASCADE,
    is_visible                                  BOOLEAN
);

CREATE TABLE IF NOT EXISTS account(
    rowid                                       TEXT PRIMARY KEY,

    is_debit                                    BOOLEAN,
    is_permanent_account                        BOOLEAN,
    name                                        TEXT,
    notes                                       TEXT,

    person_out_side_the_system_rowid            TEXT REFERENCES person_out_side_the_system(rowid) ON DELETE CASCADE,
    company_rowid                               TEXT REFERENCES company(rowid) ON DELETE CASCADE,
    company_branch_rowid                        TEXT REFERENCES company_branch(rowid) ON DELETE CASCADE,

    product_rowid                               TEXT REFERENCES product(rowid) ON DELETE CASCADE,
    is_second_hand                              BOOLEAN,
    job                                         TEXT

);

CREATE TABLE IF NOT EXISTS account_flow_type(
    rowid                                       TEXT PRIMARY KEY,

    account                                     TEXT REFERENCES account(rowid) ON DELETE CASCADE,
    company_branch                              TEXT REFERENCES company_branch(rowid) ON DELETE CASCADE,
    outflow_type                                TEXT,
    inflow_type                                 TEXT
);

CREATE TABLE IF NOT EXISTS inventory_record(
    rowid                                       TEXT PRIMARY KEY,

    account                                     TEXT REFERENCES account_flow_type(rowid) ON DELETE CASCADE,
    time                                        INTEGER,
    quantity                                    DECIMAL,
    amount                                      DECIMAL
);

CREATE TABLE IF NOT EXISTS shared_entry(
    rowid                                       TEXT PRIMARY KEY,

    writer                                      TEXT REFERENCES user(rowid) ON DELETE CASCADE,
    notes                                       TEXT
);

CREATE TABLE IF NOT EXISTS entry(
    rowid                                       TEXT PRIMARY KEY,

    writer                                      TEXT REFERENCES user(rowid) ON DELETE CASCADE,
    notes                                       TEXT,
    time                                        INTEGER,
    shared_entry_id                             TEXT REFERENCES shared_entry(rowid) ON DELETE CASCADE
);

CREATE TABLE IF NOT EXISTS double_entry(
    rowid                                       TEXT PRIMARY KEY,

    entry                                       TEXT REFERENCES entry(rowid) ON DELETE CASCADE
);

CREATE TABLE IF NOT EXISTS single_entry(
    rowid                                       TEXT PRIMARY KEY,

    double_entry                                TEXT REFERENCES double_entry(rowid) ON DELETE CASCADE,
    account                                     TEXT REFERENCES account_flow_type(rowid) ON DELETE CASCADE,
    is_debit                                    BOOLEAN,
    cost_flow_type                              TEXT,
    quantity                                    DECIMAL,
    amount                                      DECIMAL
);

CREATE TABLE IF NOT EXISTS person_attributes(
    rowid                                       TEXT PRIMARY KEY,

    person                                      TEXT REFERENCES person_out_side_the_system(rowid) ON DELETE CASCADE,
    key_                                        TEXT,
    value                                       TEXT
);

CREATE TABLE IF NOT EXISTS invoice(
    rowid                                       TEXT PRIMARY KEY,

    entry                                       TEXT REFERENCES entry(rowid) ON DELETE CASCADE,
    notes                                       TEXT,
    discount_amount                             DECIMAL,
    purchaser_user_rowid                        TEXT REFERENCES user(rowid) ON DELETE CASCADE,
    purchaser_company_branch_rowid              TEXT REFERENCES company_branch(rowid) ON DELETE CASCADE,
    purchaser_person_out_side_the_system_rowid  TEXT REFERENCES person_out_side_the_system(rowid) ON DELETE CASCADE

    -- Enforce: exactly one owner type is set
     CONSTRAINT account_owner_exclusive CHECK (
         (purchaser_user_rowid IS NOT NULL) +
         (purchaser_company_branch_rowid IS NOT NULL) +
         (purchaser_person_out_side_the_system_rowid IS NOT NULL) = 1
     )
);

CREATE TABLE IF NOT EXISTS invoice_product(
    rowid                                       TEXT PRIMARY KEY,

    invoice                                     TEXT REFERENCES invoice(rowid) ON DELETE CASCADE,
    product                                     TEXT REFERENCES product(rowid) ON DELETE CASCADE,
    quantity                                    DECIMAL,
    selling_price                               DECIMAL,
    discount_price                              DECIMAL
);

CREATE TABLE IF NOT EXISTS product_specifications(
    rowid                                       TEXT PRIMARY KEY,

    product                                     TEXT REFERENCES product(rowid) ON DELETE CASCADE,
    key_                                        TEXT,
    value                                       TEXT
);

CREATE TABLE IF NOT EXISTS my_product_on_my_hand(
    rowid                                       TEXT PRIMARY KEY,

    product                                     TEXT REFERENCES product(rowid) ON DELETE CASCADE,
    company_branch                              TEXT REFERENCES company_branch(rowid) ON DELETE CASCADE,
    is_second_hand                              BOOLEAN,
    is_visible                                  BOOLEAN,
    selling_price                               DECIMAL,
    discount_price                              DECIMAL
);

CREATE TABLE IF NOT EXISTS their_product_on_my_hand(
    rowid                                       TEXT PRIMARY KEY,

    product                                     TEXT REFERENCES product(rowid) ON DELETE CASCADE,
    company_branch                              TEXT REFERENCES company_branch(rowid) ON DELETE CASCADE,
    is_second_hand                              BOOLEAN,
    is_visible                                  BOOLEAN,
    selling_price                               DECIMAL,
    discount_price                              DECIMAL,
    buying_price                                DECIMAL,

    creditor_company_rowid                      TEXT REFERENCES company(rowid) ON DELETE CASCADE,
    creditor_company_branch_rowid               TEXT REFERENCES company_branch(rowid) ON DELETE CASCADE,
    creditor_person_out_side_the_system_rowid   TEXT REFERENCES person_out_side_the_system(rowid) ON DELETE CASCADE

    -- Enforce: exactly one owner type is set
     CONSTRAINT account_owner_exclusive CHECK (
         (creditor_company_rowid IS NOT NULL) +
         (creditor_company_branch_rowid IS NOT NULL) +
         (creditor_person_out_side_the_system_rowid IS NOT NULL) = 1
    )
);

CREATE TABLE IF NOT EXISTS product_places_for_company_branch(
    rowid                                       TEXT PRIMARY KEY,

    company_branch                              TEXT REFERENCES company_branch(rowid) ON DELETE CASCADE,
    place_name                                  TEXT,
    quantity                                    DECIMAL,

    belong_my_product_on_my_hand_rowid          TEXT REFERENCES my_product_on_my_hand(rowid) ON DELETE CASCADE,
    belong_their_product_on_my_hand_rowid       TEXT REFERENCES their_product_on_my_hand(rowid) ON DELETE CASCADE

    -- Enforce: exactly one owner type is set
    CONSTRAINT account_owner_exclusive CHECK (
        (belong_my_product_on_my_hand_rowid IS NOT NULL) +
        (belong_their_product_on_my_hand_rowid IS NOT NULL) = 1
    )
);

CREATE TABLE IF NOT EXISTS my_product_on_their_hand(
    rowid                                       TEXT PRIMARY KEY,

    product                                     TEXT REFERENCES product(rowid) ON DELETE CASCADE,
    company_branch                              TEXT REFERENCES company_branch(rowid) ON DELETE CASCADE,
    is_second_hand                              BOOLEAN,
    selling_price                               DECIMAL,

    debitor_company_rowid                       TEXT REFERENCES company(rowid) ON DELETE CASCADE,
    debitor_company_branch_rowid                TEXT REFERENCES company_branch(rowid) ON DELETE CASCADE,
    debitor_person_out_side_the_system_rowid    TEXT REFERENCES person_out_side_the_system(rowid) ON DELETE CASCADE

    -- Enforce: exactly one owner type is set
    CONSTRAINT account_owner_exclusive CHECK (
        (debitor_company_rowid IS NOT NULL) +
        (debitor_company_branch_rowid IS NOT NULL) +
        (debitor_person_out_side_the_system_rowid IS NOT NULL) = 1
    )
);

CREATE TABLE IF NOT EXISTS product_photo(
    rowid                                       TEXT PRIMARY KEY,

    product                                     TEXT REFERENCES product(rowid) ON DELETE CASCADE,
    photo                                       TEXT REFERENCES photo(rowid) ON DELETE CASCADE,
    is_visible                                  BOOLEAN
);

CREATE TABLE IF NOT EXISTS video(
    rowid                                       TEXT PRIMARY KEY,

    video                                       BLOB
);

CREATE TABLE IF NOT EXISTS product_video(
    rowid                                       TEXT PRIMARY KEY,

    product                                     TEXT REFERENCES product(rowid) ON DELETE CASCADE,
    video                                       TEXT REFERENCES video(rowid) ON DELETE CASCADE,
    is_visible                                  BOOLEAN
);

CREATE TABLE IF NOT EXISTS product_code(
    rowid                                       TEXT PRIMARY KEY,

    code                                        TEXT,

    belong_my_product_on_my_hand_rowid          TEXT REFERENCES my_product_on_my_hand(rowid) ON DELETE CASCADE,
    belong_their_product_on_my_hand_rowid       TEXT REFERENCES their_product_on_my_hand(rowid) ON DELETE CASCADE

    -- Enforce: exactly one owner type is set
    CONSTRAINT account_owner_exclusive CHECK (
        (belong_my_product_on_my_hand_rowid IS NOT NULL) +
        (belong_their_product_on_my_hand_rowid IS NOT NULL) = 1
    )
);

CREATE TABLE IF NOT EXISTS contact(
    rowid                                       TEXT PRIMARY KEY,

    belong_to                                   TEXT REFERENCES person_out_side_the_system(rowid) ON DELETE CASCADE,
    platform                                    TEXT,
    account                                     TEXT
);

CREATE TABLE IF NOT EXISTS contact_for_user(
    rowid                                       TEXT PRIMARY KEY,

    belong_to                                   TEXT REFERENCES user(rowid) ON DELETE CASCADE,
    platform                                    TEXT,
    account                                     TEXT
);

CREATE TABLE IF NOT EXISTS contact_for_company_branch(
    rowid                                       TEXT PRIMARY KEY,

    belong_to                                   TEXT REFERENCES company_branch(rowid) ON DELETE CASCADE,
    platform                                    TEXT,
    account                                     TEXT
);

CREATE TABLE IF NOT EXISTS contact_for_company(
    rowid                                       TEXT PRIMARY KEY,

    belong_to                                   TEXT REFERENCES company(rowid) ON DELETE CASCADE,
    platform                                    TEXT,
    account                                     TEXT
);

CREATE TABLE IF NOT EXISTS employees(
    rowid                                       TEXT PRIMARY KEY,

    company_branch                              TEXT REFERENCES company_branch(rowid) ON DELETE CASCADE,
    user_                                       TEXT REFERENCES user(rowid) ON DELETE CASCADE,
    salary                                      DECIMAL
);

CREATE TABLE IF NOT EXISTS employees_time(
    rowid                                       TEXT PRIMARY KEY,

    company_branch                              TEXT REFERENCES company_branch(rowid) ON DELETE CASCADE,
    user_                                       TEXT REFERENCES user(rowid) ON DELETE CASCADE,
    is_he_enter                                 BOOLEAN,
    time                                        INTEGER
);

CREATE TABLE IF NOT EXISTS access_control_for_company(
    rowid                                       TEXT PRIMARY KEY,

    data_group                                  TEXT REFERENCES company(rowid) ON DELETE CASCADE,
    user_                                       TEXT REFERENCES user(rowid) ON DELETE CASCADE,
    role                                        TEXT
);

CREATE TABLE IF NOT EXISTS access_control_for_company_branch(
    rowid                                       TEXT PRIMARY KEY,

    data_group                                  TEXT REFERENCES company_branch(rowid) ON DELETE CASCADE,
    user_                                       TEXT REFERENCES user(rowid) ON DELETE CASCADE,
    role                                        TEXT
);

CREATE TABLE IF NOT EXISTS wish_list(
    rowid                                       TEXT PRIMARY KEY,

    product                                     TEXT REFERENCES product(rowid) ON DELETE CASCADE,
    user_                                       TEXT REFERENCES user(rowid) ON DELETE CASCADE
);

CREATE TABLE IF NOT EXISTS like(
    rowid                                       TEXT PRIMARY KEY,

    product                                     TEXT REFERENCES product(rowid) ON DELETE CASCADE,
    user_                                       TEXT REFERENCES user(rowid) ON DELETE CASCADE
);

CREATE TABLE IF NOT EXISTS comment(
    rowid                                       TEXT PRIMARY KEY,

    product                                     TEXT REFERENCES product(rowid) ON DELETE CASCADE,
    user_                                       TEXT REFERENCES user(rowid) ON DELETE CASCADE,
    comment                                     TEXT,
    reply_on                                    TEXT REFERENCES comment(rowid) ON DELETE CASCADE
);

CREATE TABLE IF NOT EXISTS shopping_list(
    rowid                                       TEXT PRIMARY KEY,

    company_branch                              TEXT REFERENCES company_branch(rowid) ON DELETE CASCADE,
    time                                        INTEGER,
    location_latitude                           DECIMAL(9,6)  CHECK (location_latitude BETWEEN -90 AND 90),
    location_longitude                          DECIMAL(10,6)  CHECK (location_longitude BETWEEN -180 AND 180),
    shipping_cost                               DECIMAL,
    notes                                       TEXT,
    discount_amount                             DECIMAL,

    user_rowid                                  TEXT REFERENCES user(rowid) ON DELETE CASCADE,
    company_branch_rowid                        TEXT REFERENCES company_branch(rowid) ON DELETE CASCADE

    -- Enforce: exactly one owner type is set
    CONSTRAINT account_owner_exclusive CHECK (
        (user_rowid IS NOT NULL) +
        (company_branch_rowid IS NOT NULL) = 1
    )
);

CREATE TABLE IF NOT EXISTS shopping_list_record(
    rowid                                       TEXT PRIMARY KEY,

    shopping_list                               TEXT REFERENCES shopping_list(rowid) ON DELETE CASCADE,
    quantity                                    DECIMAL,
    at_price                                    DECIMAL,
    at_discount                                 DECIMAL,

    product_my_product_on_my_hand_rowid         TEXT REFERENCES my_product_on_my_hand(rowid) ON DELETE CASCADE,
    product_their_product_on_my_hand_rowid      TEXT REFERENCES their_product_on_my_hand(rowid) ON DELETE CASCADE

    -- Enforce: exactly one owner type is set
    CONSTRAINT account_owner_exclusive CHECK (
        (product_my_product_on_my_hand_rowid IS NOT NULL) +
        (product_their_product_on_my_hand_rowid IS NOT NULL) = 1
    )
);

CREATE TABLE IF NOT EXISTS account_translation(
    rowid                                       TEXT PRIMARY KEY,

    company_branch                              TEXT REFERENCES company_branch(rowid) ON DELETE CASCADE,
    account                                     TEXT REFERENCES account(rowid) ON DELETE CASCADE,
    name                                        TEXT
);

CREATE TABLE IF NOT EXISTS notes_receivable(
    rowid                                       TEXT PRIMARY KEY,

    notes                                       TEXT
);

CREATE TABLE IF NOT EXISTS triple_entry_for_notes_receivable(
    rowid                                       TEXT PRIMARY KEY,

    from_                                       TEXT REFERENCES company_branch(rowid) ON DELETE CASCADE,
    to_                                         TEXT REFERENCES company_branch(rowid) ON DELETE CASCADE,
    writer                                      TEXT REFERENCES user(rowid) ON DELETE CASCADE,
    notes_receivable                            TEXT REFERENCES notes_receivable(rowid) ON DELETE CASCADE,
    quantity                                    DECIMAL,
    time                                        INTEGER
);

CREATE TABLE IF NOT EXISTS notes_receivable_users(
    rowid                                       TEXT PRIMARY KEY,

    user_                                       TEXT REFERENCES user(rowid) ON DELETE CASCADE,
    notes_receivable                            TEXT REFERENCES notes_receivable(rowid) ON DELETE CASCADE
);

CREATE TABLE IF NOT EXISTS package(
    rowid                                       TEXT PRIMARY KEY,

    location_latitude                           DECIMAL(9,6)  CHECK (location_latitude BETWEEN -90 AND 90),
    location_longitude                          DECIMAL(10,6)  CHECK (location_longitude BETWEEN -180 AND 180),
    invoice                                     TEXT REFERENCES invoice(rowid) ON DELETE CASCADE,
    amount_with_shipment_price                  DECIMAL,
    compensation_amount                         DECIMAL,
    volume_in_kg                                DECIMAL,
    weight_in_litre                             DECIMAL
);

CREATE TABLE IF NOT EXISTS triple_entry_for_package(
    rowid                                       TEXT PRIMARY KEY,

    from_                                       TEXT REFERENCES company_branch(rowid) ON DELETE CASCADE,
    to_                                         TEXT REFERENCES company_branch(rowid) ON DELETE CASCADE,
    writer                                      TEXT REFERENCES user(rowid) ON DELETE CASCADE,
    package                                     TEXT REFERENCES package(rowid) ON DELETE CASCADE,
    time                                        INTEGER
);

CREATE TABLE IF NOT EXISTS write_cache_transactions_input(
    txn_number                                  INTEGER PRIMARY KEY,

    user_                                       TEXT REFERENCES user(rowid) ON DELETE CASCADE,
    txn                                         BLOB
);

CREATE TABLE IF NOT EXISTS write_cache_transactions_result(
    txn_number                                  INTEGER PRIMARY KEY,

    user_                                       TEXT REFERENCES user(rowid) ON DELETE CASCADE,
    txn                                         BLOB
);
