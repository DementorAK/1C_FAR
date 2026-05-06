/// Known 1C metadata type UUIDs and their display names.
///
/// Sources: v8unpack metadata_types.py, cf_formats.md
///
/// Two categories:
/// - Top-level group types (appear in CF root): Catalogs, Documents, etc.
/// - Subordinate types (appear inside objects): Forms, Templates, Commands, etc.

/// Resolve a Type UUID to a human-readable display name for VFS folders.
/// Returns None for unknown UUIDs.
pub fn metadata_type_name(uuid: &str) -> Option<&'static str> {
    match uuid {
        // ===== Subordinate object types (inside EPF/ERF/CF objects) =====

        // Forms (various object contexts)
        "d5b0e5ed-256d-401c-9c36-f630cafd8a62" => Some("Forms"),              // DataProcessor form
        "a3b368c0-29e2-11d6-a3c7-0050bae0a776" => Some("Forms"),              // Report form
        "fdf816d2-1ead-11d5-b975-0050bae0a95d" => Some("Forms"),              // Catalog form
        "fb880e93-47d7-4127-9357-a20e69c17545" => Some("Forms"),              // Document form
        "ec81ad10-ca07-11d5-b9a5-0050bae0a95d" => Some("Forms"),              // DocumentJournal form
        "5372e285-03db-4f8c-8565-fe56f1aea40e" => Some("Forms"),              // ChartOfAccounts form
        "eb2b78a8-40a6-4b7e-b1b3-6ca9966cbc94" => Some("Forms"),              // ChartOfCharacteristicTypes form
        "a7f8f92a-7a4b-484b-937e-42d242e64144" => Some("Forms"),              // ChartOfCalculationTypes form
        "13134204-f60b-11d5-a3c7-0050bae0a776" => Some("Forms"),              // InformationRegister form
        "b64d9a44-1642-11d6-a3c7-0050bae0a776" => Some("Forms"),              // AccumulationRegister form
        "d3b5d6eb-4ea2-4610-a3e2-624d4e815934" => Some("Forms"),              // AccountingRegister form
        "a2cb086c-db98-43e4-a1a9-0760ab048f8d" => Some("Forms"),              // CalculationRegister form
        "87c509ab-3d38-4d67-b379-aca796298578" => Some("Forms"),              // ExchangePlan form
        "3f7a8120-b71a-4265-98bf-4d9bc09b7719" => Some("Forms"),              // BusinessProcess form
        "3f58cbfb-4172-4e49-be49-561a579bb38b" => Some("Forms"),              // Task form
        "00867c40-06b1-11d6-a3c7-0050bae0a776" => Some("Forms"),              // FilterCriterion form
        "33f2e54b-37ce-4a7a-a569-b648d7aa4634" => Some("Forms"),              // Enum form
        "b8533c0c-2342-4db3-91a2-c2b08cbf6b23" => Some("Forms"),              // SettingsStorage form
        "3448e506-5add-4fce-a604-7305466b2d8e" => Some("Forms"),              // ExternalDataSourceCube form
        "17816ebc-4068-496e-adc4-8879945a832f" => Some("Forms"),              // ExternalDataSourceTable form

        // Templates
        "3daea016-69b7-4ed4-9453-127911372fe6" => Some("Templates"),

        // Commands
        "45556acb-826a-4f73-898a-6025fc9536e1" => Some("Commands"),            // DataProcessor command
        "e7ff38c0-ec3c-47a0-ae90-20c73ca72246" => Some("Commands"),            // Report command
        "4fe87c89-9ad4-43f6-9fdb-9dc83b3879c6" => Some("Commands"),            // Catalog command
        "b544fc6a-2ba3-4885-8fb2-cb289fb6d65e" => Some("Commands"),            // Document command
        "a49a35ce-120a-4c80-8eea-b0618479cd70" => Some("Commands"),            // DocumentJournal command
        "0df30176-6865-4787-9fc8-609eb144174f" => Some("Commands"),            // ChartOfAccounts command
        "95b5e1d4-abfa-4a16-818d-a5b07b7d3f73" => Some("Commands"),            // ChartOfCharacteristicTypes command
        "2e90c75b-2f0c-4899-a7d4-5426eaefc96e" => Some("Commands"),            // ChartOfCalculationTypes command
        "b44ba719-945c-445c-8aab-1088fa4df16e" => Some("Commands"),            // InformationRegister command
        "99f328af-a77f-4572-a2d8-80ed20c81890" => Some("Commands"),            // AccumulationRegister command
        "7162da60-f7fe-4d78-ad5d-e31700f9af18" => Some("Commands"),            // AccountingRegister command
        "acdf0f11-2d59-4e37-9945-c6721871a8fe" => Some("Commands"),            // CalculationRegister command
        "d5207c64-11d5-4d46-bba2-55b7b07ff4eb" => Some("Commands"),            // ExchangePlan command
        "7a3e533c-f232-40d5-a932-6a311d2480bf" => Some("Commands"),            // BusinessProcess command
        "f27c2152-a2c9-4c30-adb1-130f5eb2590f" => Some("Commands"),            // Task command
        "23fa3b84-220a-40e9-8331-e588bed87f7d" => Some("Commands"),            // FilterCriterion command
        "6d8d73a7-ba29-401d-9032-3872ec2d6433" => Some("Commands"),            // Enum command
        "4ee40ec7-3469-439f-adb4-aa26ce2d3ec3" => Some("Commands"),            // ExternalDataSourceCube command
        "5bb6f09e-5d80-41f6-8070-9faa4d15b69b" => Some("Commands"),            // ExternalDataSourceTable command

        // Attributes & TabularSections
        "113baac0-32b7-4586-b481-2292f9e42e05" => Some("Attributes"),
        "45e46cbc-3e24-4165-8b7b-cc98a6f80211" => Some("Attributes"),          // Document attributes
        "31182525-9346-4595-81f8-6f91a72ebfc6" => Some("TabularSections"),
        "2bcef0d1-0981-11d6-b9b8-0050bae0a95d" => Some("TabularSections"),     // EPF tabular section
        "45e46cbc-8fb5-4835-b641-a1e12727181c" => Some("Columns"),             // Journal columns

        // Form attributes
        "ec6bb5e5-b7a8-4d75-bec9-658107a699cf" => Some("FormAttributes"),

        // Recalculations
        "274bf899-db0e-4df6-8ab5-67bf6371ec0b" => Some("Recalculations"),

        // ===== Top-level configuration types (CF root groups) =====

        // Top-level group UUIDs from cf_formats.md §1
        "9cd510ce-abfc-11d4-9434-004095e12fc7" => Some("Configuration"),
        "9cd510cd-abfc-11d4-9434-004095e12fc7" => Some("Configuration"),       // alt
        "0195e80c-b157-11d4-9435-004095e12fc7" => Some("Constants"),
        "0195e808-b390-11d4-940f-008048da11f9" => Some("Constants"),           // alt
        "cf4abea6-37b2-11d4-940f-008048da11f9" => Some("Catalogs"),
        "cf4abea7-37b2-11d4-940f-008048da11f9" => Some("Catalogs"),           // alt
        "061d872a-5787-460e-95ac-ed74ea3a3e84" => Some("Documents"),
        "e3687359-9aa8-45e8-9b58-e6661e524fd0" => Some("Documents"),          // alt
        "631b75a0-29e2-11d6-a3c7-0050bae0a776" => Some("Reports"),
        "e3bc57a5-c3f2-458e-89a5-19a9e342410a" => Some("Reports"),            // alt
        "bf845118-327b-4682-b5c6-285d2a0eb296" => Some("DataProcessors"),
        "84f1eb25-06ab-445a-8b89-9a2eb242cecd" => Some("DataProcessors"),     // alt
        "f6a80749-5ad7-400b-8519-39dc5dff2542" => Some("Enums"),
        "0fe48980-252d-11d6-a3c7-0050bae0a776" => Some("CommonModules"),
        "c3831ec8-d8d5-4f93-8a22-f9bfae07327f" => Some("CommonModules"),      // ExternalDataProcessor type
        "07ee8426-87f1-11d5-b99c-0050bae0a95d" => Some("CommonForms"),
        "141b714b-2d33-41bb-9878-db8ec7556770" => Some("CommonForms"),        // alt
        "0c89c792-16c3-11d5-b96b-0050bae0a95d" => Some("CommonTemplates"),
        "1bb0e972-74ba-44f2-9596-f3bb30090ed8" => Some("CommonTemplates"),    // alt
        "7dcd43d9-aca5-4926-b549-1842e6a4e8cf" => Some("CommonPictures"),
        "e0fba8ee-ef0f-4ca2-be75-29803027b40d" => Some("Roles"),
        "09736b02-9cac-4e3f-b4f7-d3e9576ab948" => Some("Roles"),             // v8unpack
        "02c3b28b-b8f4-41d3-8bf6-5a415ff6a422" => Some("Subsystems"),
        "37f2fa9a-b276-11d4-9435-004095e12fc7" => Some("Subsystems"),        // v8unpack
        "13134201-f60b-11d5-a3c7-0050bae0a776" => Some("InformationRegisters"),
        "13134200-f60b-11d5-a3c7-0050bae0a776" => Some("InformationRegisters"), // alt
        "b64d9a40-1642-11d6-a3c7-0050bae0a776" => Some("AccumulationRegisters"),
        "2deed9b8-0056-4ffe-a473-c20a6c32a0bc" => Some("AccountingRegisters"),
        "238e7e88-3c5f-48b2-8a3b-81ebbecb20ed" => Some("ChartOfAccounts"),
        "82a1b659-b220-4d94-a9bd-14d757b95a48" => Some("ChartOfCharacteristicTypes"),
        "30b100d6-b29f-47ac-aec7-cb8ca8a54767" => Some("ChartOfCalculationTypes"),
        "f2de87a8-64e5-45eb-a22d-b3aedab050e7" => Some("CalculationRegisters"),
        "857c4a91-e5f4-4fac-86ec-787626f1c108" => Some("ExchangePlans"),
        "4612bd75-71b7-4a5c-8cc5-2b0b65f9fa0d" => Some("DocumentJournals"),
        "f5a8cb0c-a99f-4315-bb02-e2213e2bb9e5" => Some("DocumentJournals"),  // alt
        "22b271dc-db00-47f5-a4f6-86d1ee55bc29" => Some("BusinessProcesses"),
        "fcd3404e-1523-48ce-9bc0-ecdb822684a1" => Some("BusinessProcesses"), // v8unpack
        "3e63355c-1378-4953-be9b-1deb5fb6bec5" => Some("Tasks"),
        "36a8e346-9aaa-4af9-bdbd-83be3c177977" => Some("DocumentNumerators"),
        "bc587f20-35d9-11d6-a3c7-0050bae0a776" => Some("Sequences"),
        "3e7bfcc0-067d-11d6-a3c7-0050bae0a776" => Some("FilterCriteria"),
        "46b4cd97-fd13-4eaa-aba2-3bddd7699218" => Some("SettingsStorages"),
        "1c57eabe-7349-44b3-b1de-ebfeab67b47d" => Some("CommandGroups"),
        "15794563-ccec-41f6-a83c-ec5f7b9a5bc1" => Some("CommonAttributes"),
        "2f1a5187-fb0e-4b05-9489-dc5dd6412348" => Some("CommonCommands"),
        "af547940-3268-434f-a3e7-e47d6d2638c3" => Some("FunctionalOptions"),
        "30d554db-541e-4f62-8970-a1c6dcfeb2bc" => Some("FunctionalOptionsParameters"),
        "c045099e-13b9-4fb6-9d50-fca00202971e" => Some("DefinedTypes"),
        "4e828da6-0f44-4b5b-b1c0-a2b3cfe7bdcc" => Some("EventSubscriptions"),
        "11bdaf85-d5ad-4d91-bb24-aa0eee139052" => Some("ScheduledJobs"),
        "24c43748-c938-45d0-8d14-01424a72b11e" => Some("SessionParameters"),
        "0fffc09c-8f4c-47cc-b41c-8d5c5a221d79" => Some("HTTPServices"),
        "8657032e-7740-4e1d-a3ba-5dd6e8afb78f" => Some("WebServices"),
        "d26096fb-7a5d-4df9-af63-47d04771fa9b" => Some("WSReferences"),
        "cc9df798-7c94-4616-97d2-7aa0b7bc515e" => Some("XDTOPackages"),
        "39bddf6a-0c3c-452b-921c-d99cfa1c2f1b" => Some("Interfaces"),
        // Note: Languages UUID (9cd510ce) is identical to Configuration UUID in v8unpack
        "3e5404af-6ef8-4c73-ad11-91bd2dfac4c8" => Some("Styles"),
        "58848766-36ea-4076-8800-e91eb49590d7" => Some("StyleItems"),
        "5274d9fc-9c3a-4a71-8f5e-a0db8ab23de5" => Some("ExternalDataSources"),
        "e68182ea-4237-4383-967f-90c1e3370bc7" => Some("ExternalDataSources"), // group
        "6e6dc072-b7ac-41e7-8f88-278d25b6da2a" => Some("Bots"),
        "bf3420b0-f6f9-41a0-b83a-fe9d4ab0b65d" => Some("IntegrationServices"),
        "e41aff26-25cf-4bb6-b6c1-3f478a75f374" => Some("Reports"),            // ERF root type

        // Misc subordinate
        "7e7123e0-29e2-11d6-a3c7-0050bae0a776" => Some("Commands"),           // report commands
        "b077d780-29e2-11d6-a3c7-0050bae0a776" => Some("Permissions"),

        _ => None,
    }
}

/// Check if a type is a "Forms" type (any variant).
pub fn is_form_type(uuid: &str) -> bool {
    metadata_type_name(uuid) == Some("Forms")
}

/// Check if a type is a "Templates" type.
pub fn is_template_type(uuid: &str) -> bool {
    metadata_type_name(uuid) == Some("Templates")
}

/// Known top-level MetaDataGroup UUIDs (appear as root[N][0] in CF containers).
pub fn is_metadata_group(uuid: &str) -> bool {
    matches!(uuid,
        "9cd510cd-abfc-11d4-9434-004095e12fc7"  // General
        | "9fcd25a0-4822-11d4-9414-008048da11f9" // Main
        | "e3687481-0a87-462c-a166-9f34594f9bba" // Accounting
        | "9de14907-ec23-4a07-96f0-85521cb6b53b" // Calculation
        | "51f2d5d8-ea4d-4064-8892-82951750031e" // DocFlow
        | "e68182ea-4237-4383-967f-90c1e3370bc7" // DataSource
        | "fb282519-d103-4dd3-bc12-cb271d631dfc" // Integration
    )
}
