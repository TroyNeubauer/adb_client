use std::{fs::File, io::Read, path::Path};

use crate::{
    Result, models::AdbServerCommand, server_device::ADBServerDevice, utils::check_extension_is_apk,
};

impl ADBServerDevice {
    /// Install an APK on device
    pub fn install<P: AsRef<Path>>(&mut self, apk_path: P) -> Result<()> {
        let mut apk_file = File::open(&apk_path)?;

        check_extension_is_apk(&apk_path)?;

        let file_size = apk_file.metadata()?.len();

        self.install_reader(&mut apk_file, file_size)?;

        log::info!(
            "APK file {} successfully installed",
            apk_path.as_ref().display()
        );

        Ok(())
    }

    /// Install an APK from bytes
    pub fn install_from_bytes(&mut self, apk_bytes: &[u8]) -> Result<()> {
        let mut cursor = std::io::Cursor::new(apk_bytes);
        let file_size = apk_bytes.len() as u64;

        self.install_reader(&mut cursor, file_size)
    }

    fn install_reader(&mut self, reader: &mut impl Read, file_size: u64) -> Result<()> {
        self.set_serial_transport()?;

        self.transport
            .send_adb_request(AdbServerCommand::Install(file_size))?;

        let mut raw_connection = self.transport.get_raw_connection()?;

        std::io::copy(reader, &mut raw_connection)?;

        let mut data = [0; 1024];
        let read_amount = self.transport.get_raw_connection()?.read(&mut data)?;

        match &data[0..read_amount] {
            b"Success\n" => Ok(()),
            d => Err(crate::RustADBError::ADBRequestFailed(String::from_utf8(
                d.to_vec(),
            )?)),
        }
    }
}
