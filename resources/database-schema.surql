BEGIN TRANSACTION;
-- ------------------------------
-- OPTION
-- ------------------------------

OPTION IMPORT;

-- ------------------------------
-- TABLE: account_provider
-- ------------------------------

DEFINE TABLE account_provider TYPE ANY SCHEMALESS PERMISSIONS NONE;

DEFINE FIELD name ON account_provider TYPE string ASSERT string::len($value) > 0 PERMISSIONS FULL;

DEFINE INDEX account_provider_name ON account_provider FIELDS name;

-- ------------------------------
-- TABLE: category
-- ------------------------------

DEFINE TABLE category TYPE ANY SCHEMALESS PERMISSIONS NONE;

-- ------------------------------
-- TABLE: listing
-- ------------------------------

DEFINE TABLE listing TYPE ANY SCHEMALESS PERMISSIONS NONE;

-- ------------------------------
-- TABLE: located_in
-- ------------------------------

DEFINE TABLE located_in TYPE ANY SCHEMALESS PERMISSIONS NONE;

DEFINE FIELD in ON located_in TYPE record<listing> PERMISSIONS FULL;
DEFINE FIELD out ON located_in TYPE record<region> PERMISSIONS FULL;
DEFINE INDEX unique_listing_region ON located_in FIELDS in, out UNIQUE;

-- ------------------------------
-- TABLE: region
-- ------------------------------

DEFINE TABLE region TYPE ANY SCHEMALESS PERMISSIONS NONE;

-- ------------------------------
-- TABLE: sells
-- ------------------------------

DEFINE TABLE sells TYPE ANY SCHEMALESS PERMISSIONS NONE;

DEFINE FIELD in ON sells TYPE record<user> PERMISSIONS FULL;
DEFINE FIELD out ON sells TYPE record<listing> PERMISSIONS FULL;
DEFINE FIELD quantity ON sells TYPE int PERMISSIONS FULL;
DEFINE INDEX unique_user_listing ON sells FIELDS in, out UNIQUE;
-- ------------------------------
-- TABLE: user
-- ------------------------------

DEFINE TABLE user TYPE ANY SCHEMALESS PERMISSIONS NONE;

DEFINE FIELD avatar ON user TYPE option<string> PERMISSIONS FULL;
DEFINE FIELD created ON user TYPE datetime DEFAULT time::now() READONLY ASSERT type::is::datetime($value) PERMISSIONS FULL;
DEFINE FIELD email ON user TYPE string ASSERT string::is::email($value) PERMISSIONS FULL;
DEFINE FIELD name ON user TYPE option<string> PERMISSIONS FULL;
DEFINE FIELD type ON user TYPE string DEFAULT 'INDIVIDUAL' PERMISSIONS FULL;
DEFINE FIELD updated ON user TYPE datetime VALUE time::now() PERMISSIONS FULL;
DEFINE FIELD username ON user TYPE string ASSERT string::len($value) > 2 PERMISSIONS FULL;

DEFINE INDEX userEmailIndex ON user FIELDS email UNIQUE;

-- ------------------------------
-- TABLE: user_account
-- ------------------------------

DEFINE TABLE user_account TYPE ANY SCHEMALESS PERMISSIONS NONE;

DEFINE FIELD in ON user_account TYPE record<user> PERMISSIONS FULL;
DEFINE FIELD out ON user_account TYPE record<account_provider> PERMISSIONS FULL;
DEFINE FIELD provider_account_id ON user_account TYPE string PERMISSIONS FULL;

DEFINE INDEX unique_user_account ON user_account FIELDS in, out UNIQUE;

-- ------------------------------
-- TABLE: user_session
-- ------------------------------

DEFINE TABLE user_session TYPE ANY SCHEMALESS PERMISSIONS NONE;

DEFINE FIELD expires_at ON user_session TYPE datetime PERMISSIONS FULL;
DEFINE FIELD in ON user_session TYPE record<user> PERMISSIONS FULL;
DEFINE FIELD out ON user_session TYPE record<account_provider> PERMISSIONS FULL;
DEFINE FIELD session_token ON user_session TYPE string PERMISSIONS FULL;

DEFINE INDEX unique_session ON user_session FIELDS in, out, session_token UNIQUE;

COMMIT TRANSACTION;
