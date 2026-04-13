CREATE DATABASE IF NOT EXISTS accounting_app;
USE accounting_app;
CREATE SCHEMA IF NOT EXISTS accounting_app;

CREATE TABLE IF NOT EXISTS accounting_app.user(
    rowid                                       UUID PRIMARY KEY,

    name                                        STRING,
    id                                          STRING NOT NULL UNIQUE,
    pass                                        STRING NOT NULL
);

CREATE TABLE IF NOT EXISTS accounting_app.person_out_side_the_system(
    rowid                                       UUID PRIMARY KEY,

    name                                        STRING
);

CREATE TABLE IF NOT EXISTS accounting_app.company(
    rowid                                       UUID PRIMARY KEY,

    name                                        STRING NOT NULL,
    currency                                    STRING NOT NULL
);

CREATE TABLE IF NOT EXISTS accounting_app.company_branch(
    rowid                                       UUID PRIMARY KEY,

    company_belong                              UUID REFERENCES accounting_app.company(rowid) ON DELETE CASCADE NOT NULL,
    name                                        STRING NOT NULL,
    location_latitude                           DECIMAL(9,6) NOT NULL CHECK (location_latitude BETWEEN -90 AND 90),
    location_longitude                          DECIMAL(10,6) NOT NULL CHECK (location_longitude BETWEEN -180 AND 180),
    currency                                    STRING NOT NULL
);

CREATE TABLE IF NOT EXISTS accounting_app.photo(
    rowid                                       UUID PRIMARY KEY,

    photo                                       BYTES NOT NULL
);

CREATE TABLE IF NOT EXISTS accounting_app.product(
    rowid                                       UUID PRIMARY KEY,

    name                                        STRING,
    primary_photo                               UUID REFERENCES accounting_app.photo(rowid) ON DELETE CASCADE NOT NULL,
    is_visible                                  BOOL NOT NULL
);

CREATE TABLE IF NOT EXISTS accounting_app.account(
    rowid                                       UUID PRIMARY KEY,

    is_debit                                    BOOL NOT NULL,
    is_permanent_account                        BOOL NOT NULL,
    name                                        STRING NOT NULL,
    notes                                       STRING,

    person_out_side_the_system_rowid            UUID REFERENCES accounting_app.person_out_side_the_system(rowid) ON DELETE CASCADE,
    company_rowid                               UUID REFERENCES accounting_app.company(rowid) ON DELETE CASCADE,
    company_branch_rowid                        UUID REFERENCES accounting_app.company_branch(rowid) ON DELETE CASCADE,

    product_rowid                               UUID REFERENCES accounting_app.product(rowid) ON DELETE CASCADE,
    is_second_hand                              BOOL,
    job                                         STRING

);

CREATE TABLE IF NOT EXISTS accounting_app.account_flow_type(
    rowid                                       UUID PRIMARY KEY,

    account                                     UUID REFERENCES accounting_app.account(rowid) ON DELETE CASCADE NOT NULL,
    company_branch                              UUID REFERENCES accounting_app.company_branch(rowid) ON DELETE CASCADE NOT NULL,
    outflow_type                                STRING NOT NULL,
    inflow_type                                 STRING NOT NULL
);

CREATE TABLE IF NOT EXISTS accounting_app.inventory_record(
    rowid                                       UUID PRIMARY KEY,

    account                                     UUID REFERENCES accounting_app.account_flow_type(rowid) ON DELETE CASCADE NOT NULL,
    time                                        TIMESTAMPTZ NOT NULL,
    quantity                                    DECIMAL NOT NULL,
    amount                                      DECIMAL NOT NULL
);

CREATE TABLE IF NOT EXISTS accounting_app.shared_entry(
    rowid                                       UUID PRIMARY KEY,

    writer                                      UUID REFERENCES accounting_app.user(rowid) ON DELETE CASCADE NOT NULL,
    notes                                       STRING
);

CREATE TABLE IF NOT EXISTS accounting_app.entry(
    rowid                                       UUID PRIMARY KEY,

    writer                                      UUID REFERENCES accounting_app.user(rowid) ON DELETE CASCADE NOT NULL,
    notes                                       STRING,
    time                                        TIMESTAMPTZ NOT NULL,
    shared_entry_id                             UUID REFERENCES accounting_app.shared_entry(rowid) ON DELETE CASCADE NOT NULL
);

CREATE TABLE IF NOT EXISTS accounting_app.double_entry(
    rowid                                       UUID PRIMARY KEY,

    entry                                       UUID REFERENCES accounting_app.entry(rowid) ON DELETE CASCADE NOT NULL
);

CREATE TABLE IF NOT EXISTS accounting_app.single_entry(
    rowid                                       UUID PRIMARY KEY,

    double_entry                                UUID REFERENCES accounting_app.double_entry(rowid) ON DELETE CASCADE NOT NULL,
    account                                     UUID REFERENCES accounting_app.account_flow_type(rowid) ON DELETE CASCADE NOT NULL,
    is_debit                                    BOOL NOT NULL,
    cost_flow_type                              STRING NOT NULL,
    quantity                                    DECIMAL NOT NULL,
    amount                                      DECIMAL NOT NULL
);

CREATE TABLE IF NOT EXISTS accounting_app.person_attributes(
    rowid                                       UUID PRIMARY KEY,

    person                                      UUID REFERENCES accounting_app.person_out_side_the_system(rowid) ON DELETE CASCADE NOT NULL,
    key_                                        STRING NOT NULL,
    value                                       STRING NOT NULL
);

CREATE TABLE IF NOT EXISTS accounting_app.invoice(
    rowid                                       UUID PRIMARY KEY,

    entry                                       UUID REFERENCES accounting_app.entry(rowid) ON DELETE CASCADE NOT NULL,
    notes                                       STRING,
    discount_amount                             DECIMAL NOT NULL,
    purchaser_user_rowid                        UUID REFERENCES accounting_app.user(rowid) ON DELETE CASCADE,
    purchaser_company_branch_rowid              UUID REFERENCES accounting_app.company_branch(rowid) ON DELETE CASCADE,
    purchaser_person_out_side_the_system_rowid  UUID REFERENCES accounting_app.person_out_side_the_system(rowid) ON DELETE CASCADE

    -- Enforce: exactly one owner type is set
     CONSTRAINT account_owner_exclusive CHECK (
         (purchaser_user_rowid IS NOT NULL)::INT +
         (purchaser_company_branch_rowid IS NOT NULL)::INT +
         (purchaser_person_out_side_the_system_rowid IS NOT NULL)::INT = 1
     )
);

CREATE TABLE IF NOT EXISTS accounting_app.invoice_product(
    rowid                                       UUID PRIMARY KEY,

    invoice                                     UUID REFERENCES accounting_app.invoice(rowid) ON DELETE CASCADE NOT NULL,
    product                                     UUID REFERENCES accounting_app.product(rowid) ON DELETE CASCADE NOT NULL,
    quantity                                    DECIMAL NOT NULL,
    selling_price                               DECIMAL NOT NULL,
    discount_price                              DECIMAL NOT NULL
);

CREATE TABLE IF NOT EXISTS accounting_app.product_specifications(
    rowid                                       UUID PRIMARY KEY,

    product                                     UUID REFERENCES accounting_app.product(rowid) ON DELETE CASCADE NOT NULL,
    key_                                        STRING NOT NULL,
    value                                       STRING NOT NULL
);

CREATE TABLE IF NOT EXISTS accounting_app.my_product_on_my_hand(
    rowid                                       UUID PRIMARY KEY,

    product                                     UUID REFERENCES accounting_app.product(rowid) ON DELETE CASCADE NOT NULL,
    company_branch                              UUID REFERENCES accounting_app.company_branch(rowid) ON DELETE CASCADE NOT NULL,
    is_second_hand                              BOOL NOT NULL,
    is_visible                                  BOOL NOT NULL,
    selling_price                               DECIMAL NOT NULL,
    discount_price                              DECIMAL NOT NULL
);

CREATE TABLE IF NOT EXISTS accounting_app.their_product_on_my_hand(
    rowid                                       UUID PRIMARY KEY,

    product                                     UUID REFERENCES accounting_app.product(rowid) ON DELETE CASCADE NOT NULL,
    company_branch                              UUID REFERENCES accounting_app.company_branch(rowid) ON DELETE CASCADE NOT NULL,
    is_second_hand                              BOOL NOT NULL,
    is_visible                                  BOOL NOT NULL,
    selling_price                               DECIMAL NOT NULL,
    discount_price                              DECIMAL NOT NULL,
    buying_price                                DECIMAL NOT NULL,

    creditor_company_rowid                      UUID REFERENCES accounting_app.company(rowid) ON DELETE CASCADE,
    creditor_company_branch_rowid               UUID REFERENCES accounting_app.company_branch(rowid) ON DELETE CASCADE,
    creditor_person_out_side_the_system_rowid   UUID REFERENCES accounting_app.person_out_side_the_system(rowid) ON DELETE CASCADE

    -- Enforce: exactly one owner type is set
     CONSTRAINT account_owner_exclusive CHECK (
         (creditor_company_rowid IS NOT NULL)::INT +
         (creditor_company_branch_rowid IS NOT NULL)::INT +
         (creditor_person_out_side_the_system_rowid IS NOT NULL)::INT = 1
    )
);

CREATE TABLE IF NOT EXISTS accounting_app.product_places_for_company_branch(
    rowid                                       UUID PRIMARY KEY,

    company_branch                              UUID REFERENCES accounting_app.company_branch(rowid) ON DELETE CASCADE NOT NULL,
    place_name                                  STRING NOT NULL,
    quantity                                    DECIMAL NOT NULL,

    belong_my_product_on_my_hand_rowid          UUID REFERENCES accounting_app.my_product_on_my_hand(rowid) ON DELETE CASCADE,
    belong_their_product_on_my_hand_rowid       UUID REFERENCES accounting_app.their_product_on_my_hand(rowid) ON DELETE CASCADE

    -- Enforce: exactly one owner type is set
    CONSTRAINT account_owner_exclusive CHECK (
        (belong_my_product_on_my_hand_rowid IS NOT NULL)::INT +
        (belong_their_product_on_my_hand_rowid IS NOT NULL)::INT = 1
    )
);

CREATE TABLE IF NOT EXISTS accounting_app.my_product_on_their_hand(
    rowid                                       UUID PRIMARY KEY,

    product                                     UUID REFERENCES accounting_app.product(rowid) ON DELETE CASCADE NOT NULL,
    company_branch                              UUID REFERENCES accounting_app.company_branch(rowid) ON DELETE CASCADE NOT NULL,
    is_second_hand                              BOOL NOT NULL,
    selling_price                               DECIMAL NOT NULL,

    debitor_company_rowid                       UUID REFERENCES accounting_app.company(rowid) ON DELETE CASCADE,
    debitor_company_branch_rowid                UUID REFERENCES accounting_app.company_branch(rowid) ON DELETE CASCADE,
    debitor_person_out_side_the_system_rowid    UUID REFERENCES accounting_app.person_out_side_the_system(rowid) ON DELETE CASCADE

    -- Enforce: exactly one owner type is set
    CONSTRAINT account_owner_exclusive CHECK (
        (debitor_company_rowid IS NOT NULL)::INT +
        (debitor_company_branch_rowid IS NOT NULL)::INT +
        (debitor_person_out_side_the_system_rowid IS NOT NULL)::INT = 1
    )
);

CREATE TABLE IF NOT EXISTS accounting_app.product_photo(
    rowid                                       UUID PRIMARY KEY,

    product                                     UUID REFERENCES accounting_app.product(rowid) ON DELETE CASCADE NOT NULL,
    photo                                       UUID REFERENCES accounting_app.photo(rowid) ON DELETE CASCADE NOT NULL,
    is_visible                                  BOOL NOT NULL
);

CREATE TABLE IF NOT EXISTS accounting_app.video(
    rowid                                       UUID PRIMARY KEY,

    video                                       BYTES NOT NULL
);

CREATE TABLE IF NOT EXISTS accounting_app.product_video(
    rowid                                       UUID PRIMARY KEY,

    product                                     UUID REFERENCES accounting_app.product(rowid) ON DELETE CASCADE NOT NULL,
    video                                       UUID REFERENCES accounting_app.video(rowid) ON DELETE CASCADE NOT NULL,
    is_visible                                  BOOL NOT NULL
);

CREATE TABLE IF NOT EXISTS accounting_app.product_code(
    rowid                                       UUID PRIMARY KEY,

    code                                        STRING NOT NULL,

    belong_my_product_on_my_hand_rowid          UUID REFERENCES accounting_app.my_product_on_my_hand(rowid) ON DELETE CASCADE,
    belong_their_product_on_my_hand_rowid       UUID REFERENCES accounting_app.their_product_on_my_hand(rowid) ON DELETE CASCADE

    -- Enforce: exactly one owner type is set
    CONSTRAINT account_owner_exclusive CHECK (
        (belong_my_product_on_my_hand_rowid IS NOT NULL)::INT +
        (belong_their_product_on_my_hand_rowid IS NOT NULL)::INT = 1
    )
);

CREATE TABLE IF NOT EXISTS accounting_app.contact(
    rowid                                       UUID PRIMARY KEY,

    belong_to                                   UUID REFERENCES accounting_app.person_out_side_the_system(rowid) ON DELETE CASCADE NOT NULL,
    platform                                    STRING NOT NULL,
    account                                     STRING NOT NULL
);

CREATE TABLE IF NOT EXISTS accounting_app.contact_for_user(
    rowid                                       UUID PRIMARY KEY,

    belong_to                                   UUID REFERENCES accounting_app.user(rowid) ON DELETE CASCADE NOT NULL,
    platform                                    STRING NOT NULL,
    account                                     STRING NOT NULL
);

CREATE TABLE IF NOT EXISTS accounting_app.contact_for_company_branch(
    rowid                                       UUID PRIMARY KEY,

    belong_to                                   UUID REFERENCES accounting_app.company_branch(rowid) ON DELETE CASCADE NOT NULL,
    platform                                    STRING NOT NULL,
    account                                     STRING NOT NULL
);

CREATE TABLE IF NOT EXISTS accounting_app.contact_for_company(
    rowid                                       UUID PRIMARY KEY,

    belong_to                                   UUID REFERENCES accounting_app.company(rowid) ON DELETE CASCADE NOT NULL,
    platform                                    STRING NOT NULL,
    account                                     STRING NOT NULL
);

CREATE TABLE IF NOT EXISTS accounting_app.employees(
    rowid                                       UUID PRIMARY KEY,

    company_branch                              UUID REFERENCES accounting_app.company_branch(rowid) ON DELETE CASCADE NOT NULL,
    user_                                       UUID REFERENCES accounting_app.user(rowid) ON DELETE CASCADE NOT NULL,
    salary                                      DECIMAL NOT NULL
);

CREATE TABLE IF NOT EXISTS accounting_app.employees_time(
    rowid                                       UUID PRIMARY KEY,

    company_branch                              UUID REFERENCES accounting_app.company_branch(rowid) ON DELETE CASCADE NOT NULL,
    user_                                       UUID REFERENCES accounting_app.user(rowid) ON DELETE CASCADE NOT NULL,
    is_he_enter                                 BOOL NOT NULL,
    time                                        TIMESTAMPTZ NOT NULL
);

CREATE TABLE IF NOT EXISTS accounting_app.access_control_for_company(
    rowid                                       UUID PRIMARY KEY,

    data_group                                  UUID REFERENCES accounting_app.company(rowid) ON DELETE CASCADE NOT NULL,
    user_                                       UUID REFERENCES accounting_app.user(rowid) ON DELETE CASCADE NOT NULL,
    role                                        STRING NOT NULL
);

CREATE TABLE IF NOT EXISTS accounting_app.access_control_for_company_branch(
    rowid                                       UUID PRIMARY KEY,

    data_group                                  UUID REFERENCES accounting_app.company_branch(rowid) ON DELETE CASCADE NOT NULL,
    user_                                       UUID REFERENCES accounting_app.user(rowid) ON DELETE CASCADE NOT NULL,
    role                                        STRING NOT NULL
);

CREATE TABLE IF NOT EXISTS accounting_app.wish_list(
    rowid                                       UUID PRIMARY KEY,

    product                                     UUID REFERENCES accounting_app.product(rowid) ON DELETE CASCADE NOT NULL,
    user_                                       UUID REFERENCES accounting_app.user(rowid) ON DELETE CASCADE NOT NULL
);

CREATE TABLE IF NOT EXISTS accounting_app.like(
    rowid                                       UUID PRIMARY KEY,

    product                                     UUID REFERENCES accounting_app.product(rowid) ON DELETE CASCADE NOT NULL,
    user_                                       UUID REFERENCES accounting_app.user(rowid) ON DELETE CASCADE NOT NULL
);

CREATE TABLE IF NOT EXISTS accounting_app.comment(
    rowid                                       UUID PRIMARY KEY,

    product                                     UUID REFERENCES accounting_app.product(rowid) ON DELETE CASCADE NOT NULL,
    user_                                       UUID REFERENCES accounting_app.user(rowid) ON DELETE CASCADE NOT NULL,
    comment                                     STRING NOT NULL,
    reply_on                                    UUID REFERENCES accounting_app.comment(rowid) ON DELETE CASCADE
);

CREATE TABLE IF NOT EXISTS accounting_app.shopping_list(
    rowid                                       UUID PRIMARY KEY,

    company_branch                              UUID REFERENCES accounting_app.company_branch(rowid) ON DELETE CASCADE NOT NULL,
    time                                        TIMESTAMPTZ NOT NULL,
    location_latitude                           DECIMAL(9,6) NOT NULL CHECK (location_latitude BETWEEN -90 AND 90),
    location_longitude                          DECIMAL(10,6) NOT NULL CHECK (location_longitude BETWEEN -180 AND 180),
    shipping_cost                               DECIMAL NOT NULL,
    notes                                       STRING,
    discount_amount                             DECIMAL NOT NULL,

    user_rowid                                  UUID REFERENCES accounting_app.user(rowid) ON DELETE CASCADE,
    company_branch_rowid                        UUID REFERENCES accounting_app.company_branch(rowid) ON DELETE CASCADE

    -- Enforce: exactly one owner type is set
    CONSTRAINT account_owner_exclusive CHECK (
        (user_rowid IS NOT NULL)::INT +
        (company_branch_rowid IS NOT NULL)::INT = 1
    )
);

CREATE TABLE IF NOT EXISTS accounting_app.shopping_list_record(
    rowid                                       UUID PRIMARY KEY,

    shopping_list                               UUID REFERENCES accounting_app.shopping_list(rowid) ON DELETE CASCADE NOT NULL,
    quantity                                    DECIMAL NOT NULL,
    at_price                                    DECIMAL NOT NULL,
    at_discount                                 DECIMAL NOT NULL,

    product_my_product_on_my_hand_rowid         UUID REFERENCES accounting_app.my_product_on_my_hand(rowid) ON DELETE CASCADE,
    product_their_product_on_my_hand_rowid      UUID REFERENCES accounting_app.their_product_on_my_hand(rowid) ON DELETE CASCADE

    -- Enforce: exactly one owner type is set
    CONSTRAINT account_owner_exclusive CHECK (
        (product_my_product_on_my_hand_rowid IS NOT NULL)::INT +
        (product_their_product_on_my_hand_rowid IS NOT NULL)::INT = 1
    )
);

CREATE TABLE IF NOT EXISTS accounting_app.account_translation(
    rowid                                       UUID PRIMARY KEY,

    company_branch                              UUID REFERENCES accounting_app.company_branch(rowid) ON DELETE CASCADE NOT NULL,
    account                                     UUID REFERENCES accounting_app.account(rowid) ON DELETE CASCADE NOT NULL,
    name                                        STRING NOT NULL
);

CREATE TABLE IF NOT EXISTS accounting_app.notes_receivable(
    rowid                                       UUID PRIMARY KEY,

    notes                                       STRING NOT NULL
);

CREATE TABLE IF NOT EXISTS accounting_app.triple_entry_for_notes_receivable(
    rowid                                       UUID PRIMARY KEY,

    from_                                       UUID REFERENCES accounting_app.company_branch(rowid) ON DELETE CASCADE NOT NULL,
    to_                                         UUID REFERENCES accounting_app.company_branch(rowid) ON DELETE CASCADE NOT NULL,
    writer                                      UUID REFERENCES accounting_app.user(rowid) ON DELETE CASCADE NOT NULL,
    notes_receivable                            UUID REFERENCES accounting_app.notes_receivable(rowid) ON DELETE CASCADE NOT NULL,
    quantity                                    DECIMAL NOT NULL,
    time                                        TIMESTAMPTZ NOT NULL
);

CREATE TABLE IF NOT EXISTS accounting_app.notes_receivable_users(
    rowid                                       UUID PRIMARY KEY,

    user_                                       UUID REFERENCES accounting_app.user(rowid) ON DELETE CASCADE NOT NULL,
    notes_receivable                            UUID REFERENCES accounting_app.notes_receivable(rowid) ON DELETE CASCADE NOT NULL
);

CREATE TABLE IF NOT EXISTS accounting_app.package(
    rowid                                       UUID PRIMARY KEY,

    location_latitude                           DECIMAL(9,6) NOT NULL CHECK (location_latitude BETWEEN -90 AND 90),
    location_longitude                          DECIMAL(10,6) NOT NULL CHECK (location_longitude BETWEEN -180 AND 180),
    invoice                                     UUID REFERENCES accounting_app.invoice(rowid) ON DELETE CASCADE NOT NULL,
    amount_with_shipment_price                  DECIMAL NOT NULL,
    compensation_amount                         DECIMAL NOT NULL,
    volume_in_kg                                DECIMAL NOT NULL,
    weight_in_litre                             DECIMAL NOT NULL
);

CREATE TABLE IF NOT EXISTS accounting_app.triple_entry_for_package(
    rowid                                       UUID PRIMARY KEY,

    from_                                       UUID REFERENCES accounting_app.company_branch(rowid) ON DELETE CASCADE NOT NULL,
    to_                                         UUID REFERENCES accounting_app.company_branch(rowid) ON DELETE CASCADE NOT NULL,
    writer                                      UUID REFERENCES accounting_app.user(rowid) ON DELETE CASCADE NOT NULL,
    package                                     UUID REFERENCES accounting_app.package(rowid) ON DELETE CASCADE NOT NULL,
    time                                        TIMESTAMPTZ NOT NULL
);

CREATE TABLE IF NOT EXISTS accounting_app.transaction_number(
    rowid                                       INT PRIMARY KEY,

    time                                        TIMESTAMPTZ NOT NULL
);
