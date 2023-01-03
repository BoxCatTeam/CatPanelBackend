use std::sync::Arc;
use std::time::{Duration, Instant};

use async_graphql::{ComplexObject, Context, Enum, Object, SimpleObject};
use crossbeam_utils::atomic::AtomicCell;
use fnv::FnvHashMap;
use smol_str::SmolStr;
use sysinfo::{
    Component, ComponentExt, Cpu, CpuExt, Disk, DiskExt, NetworkData, NetworkExt, NetworksExt,
    System, SystemExt,
};
use tokio::sync::{RwLock, RwLockReadGuard};

use crate::configure::{get_config, init_configure};

macro_rules! simple_system_info_object {
    (@$impl_name:ident; $($(#[$attr:meta])* $name:ident -> $ret:ty;)*) => {
        #[Object]
        impl $impl_name {
            $(
                $(#[$attr])*
                async fn $name(&self, ctx: &Context<'_>) -> async_graphql::Result<$ret> {
                    let system = ctx.data::<LimitedRefreshSystem>()?.system().await;
                    Ok(system.$name())
                }
            )*
        }
    };
    (@$impl_name:ident; @$refresh:ident; @$refresh_key:expr; $($(#[$attr:meta])* $name:ident -> $ret:ty;)*) => {
        #[Object]
        impl $impl_name {
            $(
                $(#[$attr])*
                async fn $name(&self, ctx: &Context<'_>) -> async_graphql::Result<$ret> {
                    Ok(ctx.data::<LimitedRefreshSystem>()?
                        .maybe_refresh_nonblocking($refresh_key, System::$refresh, System::$name)
                        .await)
                }
            )*
        }
    };
}

#[derive(SimpleObject, Default)]
#[graphql(complex)]
pub struct SystemInfo {
    /// 内存相关信息
    memory: MemoryInfo,
    /// 系统相关信息
    system: SystemInfoInner,
}

#[derive(Default)]
pub struct MemoryInfo;

#[derive(Default)]
pub struct SystemInfoInner;

#[derive(SimpleObject)]
pub struct CpuInfo {
    /// MHz
    frequency: u64,
    /// - 文档原话:
    /// 如果想要一个非零值，首先需要至少刷新两次(第一次和第二次之间的差异是 CPU 使用率的计算方式).
    cpu_usage: f32,
    /// cpu名字
    name: SmolStr,
    /// 供应商id
    vendor_id: SmolStr,
    /// 品牌
    brand: SmolStr,
}

#[derive(SimpleObject)]
pub struct NetworkInfo {
    /// 网络接口名字
    interface_name: SmolStr,
    /// 上次刷新以来接收到的字节数
    received: u64,
    /// 接收到的总字节数
    total_received: u64,
    /// 上次刷新以来发出的字节数
    transmitted: u64,
    /// 发出的总字节数
    total_transmitted: u64,
    /// 自上次刷新以来收到的数据包数
    packets_received: u64,
    /// 收到的总数据包数
    total_packets_received: u64,
    /// 自上次刷新以来发出的数据包数
    packets_transmitted: u64,
    /// 发出的总数据包数
    total_packets_transmitted: u64,

    /// 自上次刷新以来接收错误的数
    errors_on_received: u64,
    /// 接收错误的总数
    total_errors_on_received: u64,
    /// 自上次刷新以来发送错误的数
    errors_on_transmitted: u64,
    /// 发送错误的总数
    total_errors_on_transmitted: u64,
}

#[derive(SimpleObject)]
pub struct ComponentInfo {
    /// 组件的温度(摄氏度)
    temperature: f32,
    /// 组件的最高温度(摄氏度)
    max: f32,
    /// 组件停止前的最高温度(摄氏度)
    critical: Option<f32>,
    /// 组件的标签
    /// **它的格式可能会改变**
    /// 详细可查看: [sysinfo库的文档](https://docs.rs/sysinfo/latest/sysinfo/trait.ComponentExt.html#tymethod.label)
    label: SmolStr,
}

#[derive(Enum, Copy, Clone, Eq, PartialEq)]
#[allow(clippy::upper_case_acronyms)]
enum DiskType {
    HDD,
    SSD,
    Unknown,
}

#[derive(SimpleObject)]
pub struct DiskInfo {
    /// 磁盘类型
    r#type: DiskType,
    /// 磁盘名字
    /// 但磁盘名字可能含有非法unicode字符, 这个情况下会被替换为`U+FFFD`
    name: SmolStr,
    //file_system: &[u8]?????
    /// 挂载点
    mount_point: String,
    /// 总大小
    total_space: u64,
    /// 可用大小
    available_space: u64,
    /// 磁盘是否可移除
    is_removable: bool,
}

simple_system_info_object! {
    @MemoryInfo;
    @refresh_memory;
    @RefreshKey::Memory;
    /// 内存大小
    total_memory -> u64;
    /// 空闲内存
    /// 通常，free memory是指未分配的内存，而available memory是指可供（重新）使用的内存
    /// **Windows不报告此项, 因此可能为0**
    free_memory -> u64;
    /// 可用内存
    /// 通常，free memory是指未分配的内存，而available memory是指可供（重新）使用的内存
    /// **Windows和FreeBSD不报告此项, 因此可能为0**
    available_memory -> u64;
    /// 已用内存
    used_memory -> u64;
    /// swap大小
    total_swap -> u64;
    /// 空闲swap
    free_swap -> u64;
    /// 已用swap
    used_swap -> u64;
}

simple_system_info_object! {
    @SystemInfoInner;
    /// cpu物理核心数
    physical_core_count -> Option<usize>;
    /// 系统uptime (秒)
    uptime -> u64;
    /// 从UNIX epoch开始的系统启动时间
    boot_time -> u64;
    /// 系统名字
    name -> Option<String>;
    /// 系统主机名
    host_name -> Option<String>;
    /// 操作系统版本
    os_version -> Option<String>;
    /// 长的操作系统版本
    long_os_version -> Option<String>;
    /// 内核版本
    kernel_version -> Option<String>;
    /// 返回由os-release定义的distribution id
    distribution_id -> String;
}

#[ComplexObject]
impl SystemInfo {
    /// 全局cpu信息(综合全部cpu)
    async fn cpu(&self, ctx: &Context<'_>) -> async_graphql::Result<CpuInfo> {
        Ok(ctx
            .data::<LimitedRefreshSystem>()?
            .maybe_refresh_nonblocking(RefreshKey::Cpu, System::refresh_cpu, |system| {
                system.global_cpu_info().into()
            })
            .await)
    }

    /// 全部cpu的信息
    async fn cpus(&self, ctx: &Context<'_>) -> async_graphql::Result<Vec<CpuInfo>> {
        Ok(ctx
            .data::<LimitedRefreshSystem>()?
            .maybe_refresh_nonblocking(RefreshKey::Cpu, System::refresh_cpu, |system| {
                system.cpus().iter().map(Into::into).collect()
            })
            .await)
    }

    /// 全部网络接口的信息
    async fn networks(&self, ctx: &Context<'_>) -> async_graphql::Result<Vec<NetworkInfo>> {
        Ok(ctx
            .data::<LimitedRefreshSystem>()?
            .maybe_refresh_nonblocking(
                RefreshKey::Network,
                |system| {
                    system.refresh_networks_list();
                    system.refresh_networks();
                },
                |system| system.networks().iter().map(Into::into).collect(),
            )
            .await)
    }

    /// 使用网络接口名字查询单个网络接口的信息
    async fn network(
        &self,
        ctx: &Context<'_>,
        #[graphql(desc = "网络接口名字")] interface_name: SmolStr,
    ) -> async_graphql::Result<Option<NetworkInfo>> {
        Ok(ctx
            .data::<LimitedRefreshSystem>()?
            .maybe_refresh_nonblocking(
                RefreshKey::Network,
                |system| {
                    system.refresh_networks_list();
                    system.refresh_networks();
                },
                move |system| {
                    system
                        .networks()
                        .iter()
                        .find(|(name, _)| name == &interface_name)
                        .map(Into::into)
                },
            )
            .await)
    }

    /// 全部组件信息
    /// **Linux**: 虚拟linux系统(例如docker, wsl等)不公开这些信息，所以在这些系统上组件信息可能会丢失或者错误
    /// **Windows**: 需要管理员权限
    async fn components(&self, ctx: &Context<'_>) -> async_graphql::Result<Vec<ComponentInfo>> {
        Ok(ctx
            .data::<LimitedRefreshSystem>()?
            .maybe_refresh_nonblocking(
                RefreshKey::Component,
                |system| {
                    system.refresh_components_list();
                    system.refresh_components();
                },
                |system| system.components().iter().map(Into::into).collect(),
            )
            .await)
    }

    /// 全部磁盘信息
    async fn disks(&self, ctx: &Context<'_>) -> async_graphql::Result<Vec<DiskInfo>> {
        Ok(ctx
            .data::<LimitedRefreshSystem>()?
            .maybe_refresh_nonblocking(
                RefreshKey::Disk,
                |system| {
                    system.refresh_disks_list();
                    system.refresh_disks();
                },
                |system| system.disks().iter().map(Into::into).collect(),
            )
            .await)
    }

    /// 使用磁盘名字查询单个磁盘的信息
    /// 但是磁盘名字可能包含非法unicode, 所以还提供了`disk_by_mount_point`方法供可能需要时使用
    async fn disk_by_name(
        &self,
        ctx: &Context<'_>,
        #[graphql(desc = "磁盘名字")] disk_name: SmolStr,
    ) -> async_graphql::Result<Option<DiskInfo>> {
        Ok(ctx
            .data::<LimitedRefreshSystem>()?
            .maybe_refresh_nonblocking(
                RefreshKey::Disk,
                |system| {
                    system.refresh_disks_list();
                    system.refresh_disks();
                },
                move |system| {
                    system
                        .disks()
                        .iter()
                        .find(|disk| *disk.name().to_string_lossy() == disk_name)
                        .map(Into::into)
                },
            )
            .await)
    }

    /// 使用磁盘挂载点查询单个磁盘的信息
    async fn disk_by_mount_point(
        &self,
        ctx: &Context<'_>,
        #[graphql(desc = "磁盘挂载点")] mount_point: SmolStr,
    ) -> async_graphql::Result<Option<DiskInfo>> {
        Ok(ctx
            .data::<LimitedRefreshSystem>()?
            .maybe_refresh_nonblocking(
                RefreshKey::Disk,
                |system| {
                    system.refresh_disks_list();
                    system.refresh_disks();
                },
                move |system| {
                    system
                        .disks()
                        .iter()
                        .find(|disk| disk.mount_point().display().to_string() == mount_point)
                        .map(Into::into)
                },
            )
            .await)
    }
}

impl<'a> From<&'a Cpu> for CpuInfo {
    fn from(cpu: &'a Cpu) -> Self {
        CpuInfo {
            frequency: cpu.frequency(),
            cpu_usage: cpu.cpu_usage(),
            name: cpu.name().into(),
            vendor_id: cpu.vendor_id().into(),
            brand: cpu.brand().into(),
        }
    }
}

impl<'a> From<(&'a String, &'a NetworkData)> for NetworkInfo {
    fn from((name, data): (&'a String, &'a NetworkData)) -> Self {
        NetworkInfo {
            interface_name: name.into(),
            received: data.received(),
            total_received: data.total_received(),
            transmitted: data.transmitted(),
            total_transmitted: data.total_transmitted(),
            packets_received: data.packets_received(),
            total_packets_received: data.total_packets_received(),
            packets_transmitted: data.packets_transmitted(),
            total_packets_transmitted: data.total_packets_transmitted(),
            errors_on_received: data.errors_on_received(),
            total_errors_on_received: data.total_errors_on_received(),
            errors_on_transmitted: data.errors_on_transmitted(),
            total_errors_on_transmitted: data.total_errors_on_transmitted(),
        }
    }
}

impl<'a> From<&'a Component> for ComponentInfo {
    fn from(component: &'a Component) -> Self {
        ComponentInfo {
            temperature: component.temperature(),
            max: component.max(),
            critical: component.critical(),
            label: component.label().into(),
        }
    }
}

impl From<sysinfo::DiskType> for DiskType {
    fn from(ty: sysinfo::DiskType) -> Self {
        match ty {
            sysinfo::DiskType::HDD => DiskType::HDD,
            sysinfo::DiskType::SSD => DiskType::SSD,
            sysinfo::DiskType::Unknown(_) => DiskType::Unknown,
        }
    }
}

impl<'a> From<&'a Disk> for DiskInfo {
    fn from(disk: &'a Disk) -> Self {
        DiskInfo {
            r#type: disk.type_().into(),
            name: disk.name().to_string_lossy().into(),
            mount_point: disk.mount_point().display().to_string(),
            total_space: disk.total_space(),
            available_space: disk.available_space(),
            is_removable: disk.is_removable(),
        }
    }
}

#[derive(Copy, Clone, Hash, PartialEq, Eq)]
pub enum RefreshKey {
    Cpu,
    Memory,
    Network,
    Disk,
    Component,
}

pub struct LimitedRefreshSystem {
    system: Arc<RwLock<System>>,
    last_refresh: FnvHashMap<RefreshKey, AtomicCell<Instant>>,
    limit: Duration,
}

fn new_last_refresh_map() -> FnvHashMap<RefreshKey, AtomicCell<Instant>> {
    let now = || AtomicCell::new(Instant::now());
    let mut map = FnvHashMap::default();
    map.insert(RefreshKey::Cpu, now());
    map.insert(RefreshKey::Memory, now());
    map.insert(RefreshKey::Network, now());
    map.insert(RefreshKey::Disk, now());
    map.insert(RefreshKey::Component, now());
    map
}

impl LimitedRefreshSystem {
    pub fn new() -> Self {
        LimitedRefreshSystem {
            system: Arc::new(RwLock::new(System::new_all())),
            last_refresh: new_last_refresh_map(),
            limit: get_config().http.system_info_refresh_limit,
        }
    }

    #[inline]
    pub async fn system(&self) -> RwLockReadGuard<'_, System> {
        self.system.read().await
    }

    async fn maybe_refresh(
        &self,
        key: RefreshKey,
        f: impl FnOnce(&mut System),
    ) -> RwLockReadGuard<'_, System> {
        if self.safe_get_last_refresh_unchecked(key).load().elapsed() > self.limit {
            let mut system = self.system.write().await;
            f(&mut system);
            self.safe_get_last_refresh_unchecked(key)
                .store(Instant::now());
            system.downgrade()
        } else {
            self.system().await
        }
    }

    async fn maybe_refresh_nonblocking<R>(
        &self,
        key: RefreshKey,
        refresh_fn: impl FnOnce(&mut System) + Send + 'static,
        read_fn: impl FnOnce(&System) -> R + Send + 'static,
    ) -> R
    where
        R: Send + 'static,
    {
        if self.safe_get_last_refresh_unchecked(key).load().elapsed() > self.limit {
            let system = self.system.clone();
            let ret = tokio::task::spawn_blocking(move || {
                let mut system = system.blocking_write();
                refresh_fn(&mut system);
                read_fn(&system.downgrade())
            })
            .await
            // join error是因为task里面panic或者task被abort了
            // 而task里的panic的情况下应该与`maybe_refresh`保持一致
            .expect("maybe_refresh_nonblocking");
            self.safe_get_last_refresh_unchecked(key)
                .store(Instant::now());
            ret
        } else {
            read_fn(&*self.system().await)
        }
    }

    // SAFETY: 全部RefreshKey都应该在hashmap里面
    #[inline]
    fn safe_get_last_refresh_unchecked(&self, key: RefreshKey) -> &AtomicCell<Instant> {
        unsafe { self.last_refresh.get(&key).unwrap_unchecked() }
    }
}

// 基于以下简单(不是很合理)的测试, 使用`maybe_refresh_nonblocking`几乎总是比`maybe_refresh`快3倍以上
// 证明`System::refresh_xxx`比较耗时的操作，所以改成使用`maybe_refresh_nonblocking`
#[tokio::test]
async fn test_nonblocking_refresh() -> anyhow::Result<()> {
    init_configure()?;
    let system = LimitedRefreshSystem::new();

    let now = Instant::now();
    let total_memory = system
        .maybe_refresh(RefreshKey::Memory, |system| {
            system.refresh_cpu();
            system.refresh_memory();
            system.refresh_networks_list();
            system.refresh_networks();
        })
        .await
        .total_memory();
    dbg!(now.elapsed());

    let now = Instant::now();
    let total_memory2 = system
        .maybe_refresh_nonblocking(
            RefreshKey::Memory,
            |system| {
                system.refresh_cpu();
                system.refresh_memory();
                system.refresh_networks_list();
                system.refresh_networks();
            },
            System::total_memory,
        )
        .await;
    dbg!(now.elapsed());

    assert_eq!(total_memory, total_memory2);
    Ok(())
}
