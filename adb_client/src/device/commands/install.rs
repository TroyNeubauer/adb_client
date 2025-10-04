use std::{fs::File, io::Read, path::Path};

use crate::{
    ADBMessageTransport, Result,
    device::{MessageWriter, adb_message_device::ADBMessageDevice},
    utils::check_extension_is_apk,
};

impl<T: ADBMessageTransport> ADBMessageDevice<T> {
    pub(crate) fn install(&mut self, apk_path: &dyn AsRef<Path>) -> Result<()> {
        let mut apk_file = File::open(apk_path)?;
        check_extension_is_apk(apk_path)?;

        let file_size = apk_file.metadata()?.len();
        self.install_reader(&mut apk_file, file_size)?;
        log::info!(
            "APK file {} successfully installed",
            apk_path.as_ref().display()
        );
        Ok(())
    }

    pub(crate) fn install_reader(&mut self, reader: &mut impl Read, file_size: u64) -> Result<()> {
        self.open_session(format!("exec:cmd package 'install' -S {file_size}\0").as_bytes())?;

        let transport = self.get_transport().clone();

        let mut writer = MessageWriter::new(transport, self.get_local_id()?, self.get_remote_id()?);

        std::io::copy(reader, &mut writer)?;

        let final_status = self.get_transport_mut().read_message()?;

        match final_status.into_payload().as_slice() {
            b"Success\n" => Ok(()),
            d => Err(crate::RustADBError::ADBRequestFailed(String::from_utf8(
                d.to_vec(),
            )?)),
        }
    }

    pub(crate) fn install_from_bytes(&mut self, apk_bytes: &[u8]) -> Result<()> {
        let mut cursor = std::io::Cursor::new(apk_bytes);
        let file_size = apk_bytes.len() as u64;

        self.install_reader(&mut cursor, file_size)
    }
}
