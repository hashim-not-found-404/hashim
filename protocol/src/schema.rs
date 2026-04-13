use macros::make_schema;
pub struct Key {
    table: u16,
    field: u8,
    record: u64,
}

impl Key {
    pub fn new(table_and_field: TableAndField, record: u64) -> Vec<u8> {
        Self {
            table: table_and_field.table,
            field: table_and_field.field,
            record,
        }
        .encode()
    }

    fn encode(&self) -> Vec<u8> {
        let mut bytes = Vec::with_capacity(11);
        bytes.extend_from_slice(&self.table.to_be_bytes());
        bytes.push(self.field);
        bytes.extend_from_slice(&self.record.to_be_bytes());
        bytes
    }

    pub fn decode(bytes: Vec<u8>) -> Self {
        Self {
            table: u16::from_be_bytes([bytes[0], bytes[1]]),
            field: bytes[2],
            record: u64::from_be_bytes(bytes[3..11].try_into().unwrap()),
        }
    }
}

pub type Bytes = Vec<u8>;
pub type ArrayOfCompanyBranch = Vec<TableCompanyBranch>;

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct InventoryRecord {
    pub time: u64,
    pub quantity: f64,
    pub amount: f64,
}

type InventoryRecords = Vec<InventoryRecord>;

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct Point {}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub enum CostFlowType {}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub enum Currency {}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub enum Role {
    Manager,
}

make_schema!(
    table 0 user
    field 0 name                                        String
    field 1 pass                                        String

    table 1 account
    field 0 is_debit                                    bool
    field 1 is_permanent_account                        bool
    field 2 name                                        String
    field 3 notes                                       String
    field 4 person                                      record<person_out_side_the_system company_branch company>
    field 5 product                                     record<product>
    field 6 is_second_hand                              bool
    field 7 job                                         String

    table 2 account_flow_type
    field 0 account                                     record<account>
    field 1 company_branch                              record<company_branch>
    field 2 outflow_type                                String
    field 3 inflow_type                                 String
    field 4 inventory_records                           InventoryRecords

    table 3 shared_entry
    field 0 writer                                      record<user>
    field 1 notes                                       String

    table 4 entry
    field 0 writer                                      record<user>
    field 1 notes                                       String
    field 2 time                                        u64
    field 3 shared_entry_id                             record<shared_entry>

    table 5 double_entry
    field 0 entry                                       record<entry>

    table 6 single_entry
    field 0 double_entry                                record<double_entry>
    field 1 account_flow_type                           record<account_flow_type>
    field 2 cost_flow_type                              CostFlowType
    field 3 quantity                                    f64
    field 4 amount                                      f64

    table 7 person_out_side_the_system
    field 0 name                                        String

    table 8 person_attributes
    field 0 person                                      record<person_out_side_the_system>
    field 1 key                                         String
    field 2 value                                       Bytes

    table 9 invoice
    field 0 entry                                       record<entry>
    field 1 notes                                       String
    field 2 purchaser                                   record<user company_branch person_out_side_the_system>
    field 3 discount_amount                             f64

    table 10 invoice_product
    field 0 invoice                                     record<invoice>
    field 1 product                                     record<product>
    field 2 quantity                                    f64
    field 3 selling_price                               f64
    field 4 discount_price                              f64

    table 11 product
    field 0 name                                        String
    field 1 primary_photo                               record<photo>
    field 2 is_visible                                  bool

    table 12 product_specifications
    field 0 product                                     record<product>
    field 1 key                                         String
    field 2 value                                       Bytes

    table 13 product_places_for_company_branch
    field 0 belong_to_product                           record<my_product_on_my_hand their_product_on_my_hand>
    field 1 company_branch                              record<company_branch>
    field 2 place_name                                  String
    field 3 quantity                                    f64

    table 14 my_product_on_my_hand
    field 0 product                                     record<product>
    field 1 company_branch                              record<company_branch>
    field 2 is_second_hand                              bool
    field 3 is_visible                                  bool
    field 4 selling_price                               f64
    field 5 discount_price                              f64

    table 15 my_product_on_their_hand
    field 0 product                                     record<product>
    field 1 company_branch                              record<company_branch>
    field 2 debitor                                     record<company company_branch person_out_side_the_system>
    field 3 is_second_hand                              bool
    field 4 selling_price                               f64

    table 16 their_product_on_my_hand
    field 0 product                                     record<product>
    field 1 company_branch                              record<company_branch>
    field 2 creditor                                    record<company company_branch person_out_side_the_system>
    field 3 is_second_hand                              bool
    field 4 is_visible                                  bool
    field 5 selling_price                               f64
    field 6 discount_price                              f64
    field 7 buying_price                                f64

    table 17 product_photo
    field 0 product                                     record<product>
    field 1 photo                                       record<photo>
    field 2 is_visible                                  bool

    table 18 product_video
    field 0 product                                     record<product>
    field 1 video                                       record<video>
    field 2 is_visible                                  bool

    table 19 product_code
    field 0 product                                     record<my_product_on_my_hand their_product_on_my_hand>
    field 1 code                                        Bytes

    table 20 photo
    field 0 photo                                       Bytes

    table 21 video
    field 0 video                                       Bytes

    table 22 contact
    field 0 belong_to                                   record<person_out_side_the_system>
    field 1 platform                                    String
    field 2 account                                     String

    table 23 contact_for_user
    field 0 belong_to                                   record<user>
    field 1 platform                                    String
    field 2 account                                     String

    table 24 contact_for_company_branch
    field 0 belong_to                                   record<company_branch>
    field 1 platform                                    String
    field 2 account                                     String

    table 25 contact_for_company
    field 0 belong_to                                   record<company>
    field 1 platform                                    String
    field 2 account                                     String

    table 26 company_branch
    field 0 company_belong                              record<company>
    field 1 manager                                     record<user>
    field 2 name                                        String
    field 3 location                                    Point
    field 4 currency                                    Currency

    table 27 company
    field 0 manager                                     record<user>
    field 1 name                                        String
    field 2 currency                                    Currency

    table 28 employees
    field 0 company_branch                              record<company_branch>
    field 1 user                                        record<user>
    field 2 salary                                      String

    table 29 employees_time
    field 0 company_branch                              record<company_branch>
    field 1 user                                        record<user>
    field 2 time_in                                     f64
    field 3 time_out                                    f64

    table 30 access_control
    field 0 user                                        record<user>
    field 1 data_group                                  record<company company_branch>
    field 2 role                                        Role

    table 31 wish_list
    field 0 product                                     record<product>
    field 1 user                                        record<user>

    table 32 like
    field 0 product                                     record<product>
    field 1 user                                        record<user>

    table 33 comment
    field 0 product                                     record<product>
    field 1 user                                        record<user>
    field 2 comment                                     String
    field 3 reply_on                                    record<comment>

    table 34 shopping_list
    field 0 company_branch                              record<company_branch>
    field 1 time                                        f64
    field 2 purchaser                                   record<user company_branch>
    field 3 location                                    Point
    field 4 shipping_cost                               f64
    field 5 notes                                       String
    field 6 discount_amount                             f64

    table 35 shopping_list_record
    field 0 shopping_list                               record<shopping_list>
    field 1 product                                     record<my_product_on_my_hand their_product_on_my_hand>
    field 2 quantity                                    f64
    field 3 at_price                                    f64
    field 4 at_discount                                 f64

    table 36 account_translation
    field 0 company_branch                              record<company_branch>
    field 1 atoti                                       record<account>
    field 2 name                                        String

    table 37 triple_entry_for_notes_receivable
    field 0 from                                        record<company_branch>
    field 1 to                                          record<company_branch>
    field 2 writer                                      record<user>
    field 3 notes_receivable                            record<notes_receivable>
    field 4 quantity                                    f64
    field 5 time                                        f64

    table 38 notes_receivable
    field 0 notes                                       String
    field 1 related_to                                  ArrayOfCompanyBranch

    table 39 triple_entry_for_package
    field 0 from                                        record<company_branch user>
    field 1 to                                          record<company_branch user>
    field 2 writer                                      record<user>
    field 3 package                                     record<package>
    field 4 time                                        f64

    table 40 package
    field 0 destination                                 Point
    field 1 invoice                                     record<invoice>
    field 2 amount_with_shipment_price                  f64
    field 3 compensation_amount                         f64
    field 4 volume_in_kg                                f64
    field 5 weight_in_litre                             f64

    table 41 transaction_number
    field 0 time                                        f64
);
