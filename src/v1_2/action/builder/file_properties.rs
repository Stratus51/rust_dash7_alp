macro_rules! build {
    ($name: ident, $test_name: ident) => {
        #[derive(Clone, Copy, Debug, PartialEq)]
        pub struct $name {
            /// Group with next action
            pub group: bool,
            /// Ask for a response (a status)
            pub resp: bool,
            pub file_id: u8,
            pub header: data::FileHeader,
        }
        crate::v1_2::action::impl_header_op!($name, group, resp, file_id, header);
        impl std::fmt::Display for $name {
            fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
                write!(
                    f,
                    "[{}{}]f({}){}",
                    if self.group { "G" } else { "-" },
                    if self.resp { "R" } else { "-" },
                    self.file_id,
                    self.header,
                )
            }
        }
        #[test]
        fn $test_name() {
            crate::test_tools::test_item(
                WriteFileProperties {
                    group: true,
                    resp: false,
                    file_id: 9,
                    header: data::FileHeader {
                        permissions: data::Permissions {
                            encrypted: true,
                            executable: false,
                            user: data::UserPermissions {
                                read: true,
                                write: true,
                                run: true,
                            },
                            guest: data::UserPermissions {
                                read: false,
                                write: false,
                                run: false,
                            },
                        },
                        properties: data::FileProperties {
                            act_en: false,
                            act_cond: data::ActionCondition::Read,
                            storage_class: data::StorageClass::Permanent,
                        },
                        alp_cmd_fid: 1,
                        interface_file_id: 2,
                        file_size: 0xDEAD_BEEF,
                        allocated_size: 0xBAAD_FACE,
                    },
                },
                &hex!("86   09   B8 13 01 02 DEADBEEF BAADFACE"),
            )
        }
    };
}
pub(crate) use build;
