mod command;
pub use command::*;

#[repr(u8)]
enum IndexCommandType {
    DirectoryUP = 0x00,
    Directory = 0x10,
    File = 0x20,
    Symlink = 0x30,
}

impl IndexCommandType {
    pub fn from_u8(input: u8) -> Option<IndexCommandType> {
        match input {
            0x00 => Some(Self::DirectoryUP),
            0x10 => Some(Self::Directory),
            0x20 => Some(Self::File),
            0x30 => Some(Self::Symlink),
            _ => None,
        }
    }

    pub fn from_command(command: &IndexCommand) -> Self {
        match command {
            IndexCommand::DirectoryUP => Self::DirectoryUP,
            IndexCommand::Directory { info: _, name: _ } => Self::Directory,
            IndexCommand::File {
                info: _,
                name: _,
                oid: _,
            } => Self::File,
            IndexCommand::Symlink {
                info: _,
                name: _,
                dest: _,
            } => Self::Symlink,
        }
    }

    pub fn to_str(&self) -> &str {
        match self {
            IndexCommandType::DirectoryUP => "DirectoryUP",
            IndexCommandType::Directory => "Directory",
            IndexCommandType::File => "File",
            IndexCommandType::Symlink => "Symlink",
        }
    }
}
