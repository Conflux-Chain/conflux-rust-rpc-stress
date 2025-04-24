use structopt::StructOpt;

// 定义命令行参数
#[derive(Debug, StructOpt)]
#[structopt(name = "rpc-bench", about = "Conflux JSON-RPC 性能测试工具")]
pub struct Opt {
    /// RPC 服务器地址
    #[structopt(short = "u", long = "url")]
    pub rpc_url: String,

    /// 并发任务数量
    #[structopt(short = "t", long = "threads", default_value = "64")]
    pub thread_count: usize,

    /// 目标请求总数
    #[structopt(short = "c", long = "count", default_value = "500000")]
    pub target_count: u64,

    /// 报告间隔 (秒)
    #[structopt(short = "i", long = "interval", default_value = "1")]
    pub report_interval: u64,

    /// 目标 QPS, 0 表示无限
    #[structopt(short = "q", long = "qps", default_value = "0")]
    pub target_qps: u64,

    /// 最大运行时间 (秒)
    #[structopt(short = "m", long = "time", default_value = "120")]
    pub max_time: u64,
}
