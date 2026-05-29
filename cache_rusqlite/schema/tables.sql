CREATE TABLE IF NOT EXISTS user(
    rowid                                       TEXT PRIMARY KEY,

    name                                        TEXT,
    id                                          TEXT NOT NULL UNIQUE,
    pass                                        TEXT NOT NULL
);

CREATE TABLE IF NOT EXISTS person_out_side_the_system(
    rowid                                       TEXT PRIMARY KEY,

    name                                        TEXT
);

CREATE TABLE IF NOT EXISTS company(
    rowid                                       TEXT PRIMARY KEY,

    name                                        TEXT NOT NULL,
    currency                                    TEXT NOT NULL
);

CREATE TABLE IF NOT EXISTS company_branch(
    rowid                                       TEXT PRIMARY KEY,

    company_belong                              TEXT REFERENCES company(rowid) ON DELETE CASCADE NOT NULL,
    name                                        TEXT NOT NULL,
    location_latitude                           DECIMAL(9,6) NOT NULL CHECK (location_latitude BETWEEN -90 AND 90),
    location_longitude                          DECIMAL(10,6) NOT NULL CHECK (location_longitude BETWEEN -180 AND 180),
    currency                                    TEXT NOT NULL
);

CREATE TABLE IF NOT EXISTS photo(
    rowid                                       TEXT PRIMARY KEY,

    photo                                       BLOB NOT NULL
);

CREATE TABLE IF NOT EXISTS product(
    rowid                                       TEXT PRIMARY KEY,

    name                                        TEXT,
    primary_photo                               TEXT REFERENCES photo(rowid) ON DELETE CASCADE NOT NULL,
    is_visible                                  BOOLEAN NOT NULL
);

CREATE TABLE IF NOT EXISTS account(
    rowid                                       TEXT PRIMARY KEY,

    is_debit                                    BOOLEAN NOT NULL,
    is_permanent_account                        BOOLEAN NOT NULL,
    name                                        TEXT NOT NULL,
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

    account                                     TEXT REFERENCES account(rowid) ON DELETE CASCADE NOT NULL,
    company_branch                              TEXT REFERENCES company_branch(rowid) ON DELETE CASCADE NOT NULL,
    outflow_type                                TEXT NOT NULL,
    inflow_type                                 TEXT NOT NULL
);

CREATE TABLE IF NOT EXISTS inventory_record(
    rowid                                       TEXT PRIMARY KEY,

    account                                     TEXT REFERENCES account_flow_type(rowid) ON DELETE CASCADE NOT NULL,
    time                                        INTEGER NOT NULL,
    quantity                                    DECIMAL NOT NULL,
    amount                                      DECIMAL NOT NULL
);

CREATE TABLE IF NOT EXISTS shared_entry(
    rowid                                       TEXT PRIMARY KEY,

    writer                                      TEXT REFERENCES user(rowid) ON DELETE CASCADE NOT NULL,
    notes                                       TEXT
);

CREATE TABLE IF NOT EXISTS entry(
    rowid                                       TEXT PRIMARY KEY,

    writer                                      TEXT REFERENCES user(rowid) ON DELETE CASCADE NOT NULL,
    notes                                       TEXT,
    time                                        INTEGER NOT NULL,
    shared_entry_id                             TEXT REFERENCES shared_entry(rowid) ON DELETE CASCADE NOT NULL
);

CREATE TABLE IF NOT EXISTS double_entry(
    rowid                                       TEXT PRIMARY KEY,

    entry                                       TEXT REFERENCES entry(rowid) ON DELETE CASCADE NOT NULL
);

CREATE TABLE IF NOT EXISTS single_entry(
    rowid                                       TEXT PRIMARY KEY,

    double_entry                                TEXT REFERENCES double_entry(rowid) ON DELETE CASCADE NOT NULL,
    account                                     TEXT REFERENCES account_flow_type(rowid) ON DELETE CASCADE NOT NULL,
    is_debit                                    BOOLEAN NOT NULL,
    cost_flow_type                              TEXT NOT NULL,
    quantity                                    DECIMAL NOT NULL,
    amount                                      DECIMAL NOT NULL
);

CREATE TABLE IF NOT EXISTS person_attributes(
    rowid                                       TEXT PRIMARY KEY,

    person                                      TEXT REFERENCES person_out_side_the_system(rowid) ON DELETE CASCADE NOT NULL,
    key_                                        TEXT NOT NULL,
    value                                       TEXT NOT NULL
);

CREATE TABLE IF NOT EXISTS invoice(
    rowid                                       TEXT PRIMARY KEY,

    entry                                       TEXT REFERENCES entry(rowid) ON DELETE CASCADE NOT NULL,
    notes                                       TEXT,
    discount_amount                             DECIMAL NOT NULL,
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

    invoice                                     TEXT REFERENCES invoice(rowid) ON DELETE CASCADE NOT NULL,
    product                                     TEXT REFERENCES product(rowid) ON DELETE CASCADE NOT NULL,
    quantity                                    DECIMAL NOT NULL,
    selling_price                               DECIMAL NOT NULL,
    discount_price                              DECIMAL NOT NULL
);

CREATE TABLE IF NOT EXISTS product_specifications(
    rowid                                       TEXT PRIMARY KEY,

    product                                     TEXT REFERENCES product(rowid) ON DELETE CASCADE NOT NULL,
    key_                                        TEXT NOT NULL,
    value                                       TEXT NOT NULL
);

CREATE TABLE IF NOT EXISTS my_product_on_my_hand(
    rowid                                       TEXT PRIMARY KEY,

    product                                     TEXT REFERENCES product(rowid) ON DELETE CASCADE NOT NULL,
    company_branch                              TEXT REFERENCES company_branch(rowid) ON DELETE CASCADE NOT NULL,
    is_second_hand                              BOOLEAN NOT NULL,
    is_visible                                  BOOLEAN NOT NULL,
    selling_price                               DECIMAL NOT NULL,
    discount_price                              DECIMAL NOT NULL
);

CREATE TABLE IF NOT EXISTS their_product_on_my_hand(
    rowid                                       TEXT PRIMARY KEY,

    product                                     TEXT REFERENCES product(rowid) ON DELETE CASCADE NOT NULL,
    company_branch                              TEXT REFERENCES company_branch(rowid) ON DELETE CASCADE NOT NULL,
    is_second_hand                              BOOLEAN NOT NULL,
    is_visible                                  BOOLEAN NOT NULL,
    selling_price                               DECIMAL NOT NULL,
    discount_price                              DECIMAL NOT NULL,
    buying_price                                DECIMAL NOT NULL,

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

    company_branch                              TEXT REFERENCES company_branch(rowid) ON DELETE CASCADE NOT NULL,
    place_name                                  TEXT NOT NULL,
    quantity                                    DECIMAL NOT NULL,

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

    product                                     TEXT REFERENCES product(rowid) ON DELETE CASCADE NOT NULL,
    company_branch                              TEXT REFERENCES company_branch(rowid) ON DELETE CASCADE NOT NULL,
    is_second_hand                              BOOLEAN NOT NULL,
    selling_price                               DECIMAL NOT NULL,

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

    product                                     TEXT REFERENCES product(rowid) ON DELETE CASCADE NOT NULL,
    photo                                       TEXT REFERENCES photo(rowid) ON DELETE CASCADE NOT NULL,
    is_visible                                  BOOLEAN NOT NULL
);

CREATE TABLE IF NOT EXISTS video(
    rowid                                       TEXT PRIMARY KEY,

    video                                       BLOB NOT NULL
);

CREATE TABLE IF NOT EXISTS product_video(
    rowid                                       TEXT PRIMARY KEY,

    product                                     TEXT REFERENCES product(rowid) ON DELETE CASCADE NOT NULL,
    video                                       TEXT REFERENCES video(rowid) ON DELETE CASCADE NOT NULL,
    is_visible                                  BOOLEAN NOT NULL
);

CREATE TABLE IF NOT EXISTS product_code(
    rowid                                       TEXT PRIMARY KEY,

    code                                        TEXT NOT NULL,

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

    belong_to                                   TEXT REFERENCES person_out_side_the_system(rowid) ON DELETE CASCADE NOT NULL,
    platform                                    TEXT NOT NULL,
    account                                     TEXT NOT NULL
);

CREATE TABLE IF NOT EXISTS contact_for_user(
    rowid                                       TEXT PRIMARY KEY,

    belong_to                                   TEXT REFERENCES user(rowid) ON DELETE CASCADE NOT NULL,
    platform                                    TEXT NOT NULL,
    account                                     TEXT NOT NULL
);

CREATE TABLE IF NOT EXISTS contact_for_company_branch(
    rowid                                       TEXT PRIMARY KEY,

    belong_to                                   TEXT REFERENCES company_branch(rowid) ON DELETE CASCADE NOT NULL,
    platform                                    TEXT NOT NULL,
    account                                     TEXT NOT NULL
);

CREATE TABLE IF NOT EXISTS contact_for_company(
    rowid                                       TEXT PRIMARY KEY,

    belong_to                                   TEXT REFERENCES company(rowid) ON DELETE CASCADE NOT NULL,
    platform                                    TEXT NOT NULL,
    account                                     TEXT NOT NULL
);

CREATE TABLE IF NOT EXISTS employees(
    rowid                                       TEXT PRIMARY KEY,

    company_branch                              TEXT REFERENCES company_branch(rowid) ON DELETE CASCADE NOT NULL,
    user_                                       TEXT REFERENCES user(rowid) ON DELETE CASCADE NOT NULL,
    salary                                      DECIMAL NOT NULL
);

CREATE TABLE IF NOT EXISTS employees_time(
    rowid                                       TEXT PRIMARY KEY,

    company_branch                              TEXT REFERENCES company_branch(rowid) ON DELETE CASCADE NOT NULL,
    user_                                       TEXT REFERENCES user(rowid) ON DELETE CASCADE NOT NULL,
    is_he_enter                                 BOOLEAN NOT NULL,
    time                                        INTEGER NOT NULL
);

CREATE TABLE IF NOT EXISTS access_control_for_company(
    rowid                                       TEXT PRIMARY KEY,

    data_group                                  TEXT REFERENCES company(rowid) ON DELETE CASCADE NOT NULL,
    user_                                       TEXT REFERENCES user(rowid) ON DELETE CASCADE NOT NULL,
    role                                        TEXT NOT NULL
);

CREATE TABLE IF NOT EXISTS access_control_for_company_branch(
    rowid                                       TEXT PRIMARY KEY,

    data_group                                  TEXT REFERENCES company_branch(rowid) ON DELETE CASCADE NOT NULL,
    user_                                       TEXT REFERENCES user(rowid) ON DELETE CASCADE NOT NULL,
    role                                        TEXT NOT NULL
);

CREATE TABLE IF NOT EXISTS wish_list(
    rowid                                       TEXT PRIMARY KEY,

    product                                     TEXT REFERENCES product(rowid) ON DELETE CASCADE NOT NULL,
    user_                                       TEXT REFERENCES user(rowid) ON DELETE CASCADE NOT NULL
);

CREATE TABLE IF NOT EXISTS like(
    rowid                                       TEXT PRIMARY KEY,

    product                                     TEXT REFERENCES product(rowid) ON DELETE CASCADE NOT NULL,
    user_                                       TEXT REFERENCES user(rowid) ON DELETE CASCADE NOT NULL
);

CREATE TABLE IF NOT EXISTS comment(
    rowid                                       TEXT PRIMARY KEY,

    product                                     TEXT REFERENCES product(rowid) ON DELETE CASCADE NOT NULL,
    user_                                       TEXT REFERENCES user(rowid) ON DELETE CASCADE NOT NULL,
    comment                                     TEXT NOT NULL,
    reply_on                                    TEXT REFERENCES comment(rowid) ON DELETE CASCADE
);

CREATE TABLE IF NOT EXISTS shopping_list(
    rowid                                       TEXT PRIMARY KEY,

    company_branch                              TEXT REFERENCES company_branch(rowid) ON DELETE CASCADE NOT NULL,
    time                                        INTEGER NOT NULL,
    location_latitude                           DECIMAL(9,6) NOT NULL CHECK (location_latitude BETWEEN -90 AND 90),
    location_longitude                          DECIMAL(10,6) NOT NULL CHECK (location_longitude BETWEEN -180 AND 180),
    shipping_cost                               DECIMAL NOT NULL,
    notes                                       TEXT,
    discount_amount                             DECIMAL NOT NULL,

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

    shopping_list                               TEXT REFERENCES shopping_list(rowid) ON DELETE CASCADE NOT NULL,
    quantity                                    DECIMAL NOT NULL,
    at_price                                    DECIMAL NOT NULL,
    at_discount                                 DECIMAL NOT NULL,

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

    company_branch                              TEXT REFERENCES company_branch(rowid) ON DELETE CASCADE NOT NULL,
    account                                     TEXT REFERENCES account(rowid) ON DELETE CASCADE NOT NULL,
    name                                        TEXT NOT NULL
);

CREATE TABLE IF NOT EXISTS notes_receivable(
    rowid                                       TEXT PRIMARY KEY,

    notes                                       TEXT NOT NULL
);

CREATE TABLE IF NOT EXISTS triple_entry_for_notes_receivable(
    rowid                                       TEXT PRIMARY KEY,

    from_                                       TEXT REFERENCES company_branch(rowid) ON DELETE CASCADE NOT NULL,
    to_                                         TEXT REFERENCES company_branch(rowid) ON DELETE CASCADE NOT NULL,
    writer                                      TEXT REFERENCES user(rowid) ON DELETE CASCADE NOT NULL,
    notes_receivable                            TEXT REFERENCES notes_receivable(rowid) ON DELETE CASCADE NOT NULL,
    quantity                                    DECIMAL NOT NULL,
    time                                        INTEGER NOT NULL
);

CREATE TABLE IF NOT EXISTS notes_receivable_users(
    rowid                                       TEXT PRIMARY KEY,

    user_                                       TEXT REFERENCES user(rowid) ON DELETE CASCADE NOT NULL,
    notes_receivable                            TEXT REFERENCES notes_receivable(rowid) ON DELETE CASCADE NOT NULL
);

CREATE TABLE IF NOT EXISTS package(
    rowid                                       TEXT PRIMARY KEY,

    location_latitude                           DECIMAL(9,6) NOT NULL CHECK (location_latitude BETWEEN -90 AND 90),
    location_longitude                          DECIMAL(10,6) NOT NULL CHECK (location_longitude BETWEEN -180 AND 180),
    invoice                                     TEXT REFERENCES invoice(rowid) ON DELETE CASCADE NOT NULL,
    amount_with_shipment_price                  DECIMAL NOT NULL,
    compensation_amount                         DECIMAL NOT NULL,
    volume_in_kg                                DECIMAL NOT NULL,
    weight_in_litre                             DECIMAL NOT NULL
);

CREATE TABLE IF NOT EXISTS triple_entry_for_package(
    rowid                                       TEXT PRIMARY KEY,

    from_                                       TEXT REFERENCES company_branch(rowid) ON DELETE CASCADE NOT NULL,
    to_                                         TEXT REFERENCES company_branch(rowid) ON DELETE CASCADE NOT NULL,
    writer                                      TEXT REFERENCES user(rowid) ON DELETE CASCADE NOT NULL,
    package                                     TEXT REFERENCES package(rowid) ON DELETE CASCADE NOT NULL,
    time                                        INTEGER NOT NULL
);

CREATE TABLE IF NOT EXISTS transaction_number(
    rowid                                       TEXT PRIMARY KEY
);
