use std::{env, fs::File, collections::HashMap, io::{self, Read}};
use std::{path::PathBuf, mem, ptr, str};
use std::cmp;
use std::ffi::{CStr, OsString};
use libc::{getuid, getpwuid_r, passwd};
use whoami;

/// Error types
#[derive(Debug)]
pub enum Error 
{
    IO(io::Error),
    Unknown,
}

impl From<io::Error> for Error 
{
    fn from(e: io::Error) -> Error 
    {
        Error::IO(e)
    }
}

#[derive(Debug)]
pub struct UserData 
{
    pub username:   String,     // User's username
    pub hostname:   String,     // User's hostname
    pub devicename: String,     // User's device name
    pub cwd:        String,     // User's current working directory. TODO: unneeded?
    pub hmd:        String,     // User's home directory
    pub desk_env:   String,     // User's desktop environment
    pub distro:     String,     // User's distro
    pub platform:   String,     // User's platform 
    pub total_memory: String,   // Total memory in human-readable form
    pub used_memory:  String    // Used memory in human-readable form
}

#[repr(C)]
#[derive(Debug)]
pub struct MemInfo 
{
    /// Total physical memory.
    pub total:   u64,
    pub free:    u64,
    pub avail:   u64,

    pub buffers: u64,
    pub cached:  u64,

    /// Total swap memory.
    pub swap_total: u64,
    pub swap_free:  u64,
}

/// Returns the user's home directory
pub fn home_dir() -> Option<PathBuf> 
{
    // Returns the home directory as specified by the HOME ..
    // .. directory 

    return std::env::var_os("HOME").or_else(|| unsafe {
        let mut buf = Vec::with_capacity(2048);
        let mut passwd: passwd = mem::zeroed();
        let mut result = ptr::null_mut();
        match getpwuid_r(getuid(), &mut passwd, buf.as_mut_ptr(), buf.capacity(), &mut result) 
        {
            0 if !result.is_null() => 
            {
                let ptr = passwd.pw_dir as *const _;
                let bytes = CStr::from_ptr(ptr).to_bytes().to_vec();
                Some(OsString::from(str::from_utf8(&bytes).unwrap()))
            },
            _ => None,
        }
    }).map(PathBuf::from);
}

// This function was adapted from sys-info by Siyu Wang (MIT-licensed)
/// Returns current memory utilization info
pub fn mem_info() -> Result<MemInfo, Error> 
{
    let mut s = String::new();
    File::open("/proc/meminfo")?.read_to_string(&mut s)?;
    let mut mem_info_map = HashMap::new();
    for line in s.lines() 
    {
        let mut split_line = line.split_whitespace();
        let label = split_line.next();
        let value = split_line.next();
        if value.is_some() && label.is_some() 
        {
            let label = label.unwrap().split(':').nth(0).ok_or(Error::Unknown)?;
            let value = value.unwrap().parse::<u64>().ok().ok_or(Error::Unknown)?;
            mem_info_map.insert(label, value);
        }
    }
    let total   = mem_info_map.get("MemTotal").ok_or(Error::Unknown)?;
    let free    = mem_info_map.get("MemFree").ok_or(Error::Unknown)?;
    let buffers = mem_info_map.get("Buffers").ok_or(Error::Unknown)?;
    let cached  = mem_info_map.get("Cached").ok_or(Error::Unknown)?;
    let avail   = mem_info_map.get("MemAvailable").map(|v| v.clone()).or_else(|| 
    {
        let sreclaimable = *mem_info_map.get("SReclaimable")?;
        let shmem = mem_info_map.get("Shmem")?;
        Some(free + buffers + cached + sreclaimable - shmem)
    }).ok_or(Error::Unknown)?;
    let swap_total = mem_info_map.get("SwapTotal").ok_or(Error::Unknown)?;
    let swap_free  = mem_info_map.get("SwapFree").ok_or(Error::Unknown)?;
    Ok(MemInfo 
    {
           total: *total,
           free: *free,
           avail,
           buffers: *buffers,
           cached: *cached,
           swap_total: *swap_total,
           swap_free: *swap_free,
       })
}

/// pretty_bytes gets a value in bytes and returns a human-readable form of it
fn pretty_bytes(num: f64) -> String {
    let negative = if num > 0_f64 { "" } else { "-" };
    let num = num.abs();
    const UNITS: &[&str] = &["B", "kB", "MB", "GB", "TB"];
    if num < 1_f64 
    {
        return format!("{}{} {}", negative, num, "B");
    }
    let v1 = (num.ln() / 1024_f64.ln()).floor() as i32;
    let exponent = cmp::min(v1, 4_i32);
    let pretty_bytes = format!("{:.2}", num / 1024_f64.powi(exponent));
    let unit:&str = UNITS[exponent as usize];
    format!("{}{} {}", negative, pretty_bytes, unit)
  }

/// get_user_data returns a new UserData structure
pub fn get_user_data() -> UserData
{
    // Current working directory
    let cwd = env::current_dir().unwrap();
    let cwd_str: String = cwd.as_os_str().to_str().unwrap().to_string();
    drop(cwd);

    // Home directory
    let hmd = home_dir().unwrap();
    let hmd_str: String = hmd.as_os_str().to_str().unwrap().to_string();
    drop(hmd);
    
    let mem_info = mem_info().unwrap();

    return UserData
    {
        username:   whoami::username(),
        hostname:   whoami::hostname(),
        devicename: whoami::devicename(),
        cwd:        cwd_str,
        hmd:        hmd_str,
        desk_env:   whoami::desktop_env().to_string(),
        distro:     whoami::distro(),
        platform:   whoami::platform().to_string(),
        total_memory: pretty_bytes((mem_info.total * 1024) as f64),
        used_memory:  pretty_bytes(((mem_info.total - mem_info.avail)*1024) as f64)
    };
}