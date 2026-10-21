//! Test-only private application hive: no keys are created in the user's HKCU.
use super::*;
use windows_sys::Win32::{
    Foundation::FreeLibrary,
    System::LibraryLoader::{GetProcAddress, LOAD_LIBRARY_SEARCH_SYSTEM32, LoadLibraryExW},
};
pub(super) struct Hive(usize);
impl Hive {
    pub fn open(path: &Path) -> Result<Self> {
        crate::platform::validate_local_path(path)?;
        let wide_path: Vec<u16> = path.as_os_str().encode_wide().chain(Some(0)).collect();
        if !path.exists() {
            // Load only Windows' own Offreg; never search the working directory.
            let module = unsafe {
                LoadLibraryExW(
                    wide("offreg.dll").as_ptr(),
                    std::ptr::null_mut(),
                    LOAD_LIBRARY_SEARCH_SYSTEM32,
                )
            };
            if module.is_null() {
                eprintln!("Offreg load: {}", std::io::Error::last_os_error());
                return Err(error());
            }
            let result = (|| {
                type Create = unsafe extern "system" fn(*mut HKEY) -> u32;
                type Save = unsafe extern "system" fn(HKEY, *const u16, u32, u32) -> u32;
                type Close = unsafe extern "system" fn(HKEY) -> u32;
                let create: Create = unsafe {
                    std::mem::transmute(
                        GetProcAddress(module, c"ORCreateHive".as_ptr().cast())
                            .ok_or_else(error)?,
                    )
                };
                let save: Save = unsafe {
                    std::mem::transmute(
                        GetProcAddress(module, c"ORSaveHive".as_ptr().cast()).ok_or_else(error)?,
                    )
                };
                let close: Close = unsafe {
                    std::mem::transmute(
                        GetProcAddress(module, c"ORCloseHive".as_ptr().cast()).ok_or_else(error)?,
                    )
                };
                let mut hive = std::ptr::null_mut();
                let created = unsafe { create(&mut hive) };
                if created != 0 {
                    return Err(error());
                }
                // Offreg's empty hive has no caller access grant. This private test
                // hive grants access inside the file; its filesystem ACL is inherited.
                type SetSecurity =
                    unsafe extern "system" fn(HKEY, u32, *const std::ffi::c_void) -> u32;
                let security: SetSecurity = unsafe {
                    std::mem::transmute(
                        GetProcAddress(module, c"ORSetKeySecurity".as_ptr().cast())
                            .ok_or_else(error)?,
                    )
                };
                let mut descriptor = std::ptr::null_mut();
                let converted = unsafe {
                    windows_sys::Win32::Security::Authorization::ConvertStringSecurityDescriptorToSecurityDescriptorW(wide("D:P(A;;KA;;;WD)").as_ptr(),1,&mut descriptor,std::ptr::null_mut())
                };
                if converted == 0 {
                    unsafe { close(hive) };
                    return Err(error());
                }
                let secured = unsafe {
                    security(
                        hive,
                        windows_sys::Win32::Security::DACL_SECURITY_INFORMATION,
                        descriptor,
                    )
                };
                unsafe { windows_sys::Win32::Foundation::LocalFree(descriptor) };
                if secured != 0 {
                    unsafe { close(hive) };
                    return Err(error());
                }
                let saved = unsafe { save(hive, wide_path.as_ptr(), 6, 1) };
                unsafe { close(hive) };
                if saved != 0 {
                    return Err(error());
                }
                Ok(())
            })();
            unsafe { FreeLibrary(module) };
            result?;
        }
        let mut handle = std::ptr::null_mut();
        let loaded = unsafe {
            RegLoadAppKeyW(
                wide_path.as_ptr(),
                &mut handle,
                KEY_READ | KEY_WRITE,
                REG_PROCESS_APPKEY,
                0,
            )
        };
        if loaded != 0 {
            return Err(error());
        }
        Ok(Self(handle as usize))
    }
    pub fn handle(&self) -> HKEY {
        self.0 as HKEY
    }
}
impl Drop for Hive {
    fn drop(&mut self) {
        unsafe {
            RegFlushKey(self.handle());
            RegCloseKey(self.handle());
        }
    }
}
use std::os::windows::ffi::OsStrExt;
