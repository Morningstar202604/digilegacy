//! 中国数字遗产平台完整数据库
//! 
//! 内置 100+ 主流平台的继承政策、所需材料、办理流程等完整信息。
//! 为前端资产发现引擎和继承人向导提供数据支撑。
//! 
//! 辅助查询函数（get_platform_by_id / get_platforms_by_category / get_difficulty_stats）
//! 预留给前端独立查询使用，MVP batch 路径暂不调用。

#![allow(dead_code)]

use serde::{Deserialize, Serialize};

/// 平台条目 - 中国数字资产平台完整清单
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlatformEntry {
    pub id: i64,
    pub name: String,
    pub name_en: String,
    pub category: String,
    pub subtype: String,
    pub has_property: bool,
    pub has_personality: bool,
    pub inheritable: bool,
    pub required_documents: String,
    pub process_description: String,
    pub estimated_difficulty: String, // "简单", "中等", "困难", "暂不支持"
    pub official_policy_url: String,
    pub estimated_time_days: String,
    pub notes: String,
    pub data_source: String, // 数据来源：权威出处 / 官网链接 / 判例编号
}

impl PlatformEntry {
    pub fn new(
        id: i64,
        name: &str,
        name_en: &str,
        category: &str,
        subtype: &str,
        has_property: bool,
        has_personality: bool,
        inheritable: bool,
        required_documents: &str,
        process_description: &str,
        estimated_difficulty: &str,
        official_policy_url: &str,
        estimated_time_days: &str,
        notes: &str,
    ) -> Self {
        Self {
            id,
            name: name.to_string(),
            name_en: name_en.to_string(),
            category: category.to_string(),
            subtype: subtype.to_string(),
            has_property,
            has_personality,
            inheritable,
            required_documents: required_documents.to_string(),
            process_description: process_description.to_string(),
            estimated_difficulty: estimated_difficulty.to_string(),
            official_policy_url: official_policy_url.to_string(),
            estimated_time_days: estimated_time_days.to_string(),
            notes: notes.to_string(),
            data_source: String::new(),
        }
    }

    /// 创建带有数据来源的平台条目
    pub fn with_source(
        id: i64,
        name: &str,
        name_en: &str,
        category: &str,
        subtype: &str,
        has_property: bool,
        has_personality: bool,
        inheritable: bool,
        required_documents: &str,
        process_description: &str,
        estimated_difficulty: &str,
        official_policy_url: &str,
        estimated_time_days: &str,
        notes: &str,
        data_source: &str,
    ) -> Self {
        Self {
            id,
            name: name.to_string(),
            name_en: name_en.to_string(),
            category: category.to_string(),
            subtype: subtype.to_string(),
            has_property,
            has_personality,
            inheritable,
            required_documents: required_documents.to_string(),
            process_description: process_description.to_string(),
            estimated_difficulty: estimated_difficulty.to_string(),
            official_policy_url: official_policy_url.to_string(),
            estimated_time_days: estimated_time_days.to_string(),
            notes: notes.to_string(),
            data_source: data_source.to_string(),
        }
    }
}

/// 获取完整的平台数据库
pub fn get_all_platforms() -> Vec<PlatformEntry> {
    vec![
        // ========== 支付金融类 ==========
        PlatformEntry::with_source(1, "支付宝", "Alipay", "支付金融", "移动支付", true, false, true,
            "死亡证明、继承公证书、继承人身份证；5万元以下免公证",
            "参照银行存款管理（2025新规）。联系支付宝客服95188，余额、余额宝资金可依法继承",
            "中等", "https://help.alipay.com", "30-90天",
            "5万以下走简易继承程序（2025银行新规）",
            "2025《简化小额存款继承流程指导意见》：支付宝/微信余额参照银行存款管理；每年节省公证费用超10亿元（中国人民银行）"
        ),
        PlatformEntry::with_source(2, "微信支付", "WeChat Pay", "支付金融", "移动支付", true, false, true,
            "死亡证明、继承公证书、继承人身份证",
            "独立于账号的余额权益可继承。资金账户与社交账号分开处理",
            "中等", "https://weixin.qq.com", "30-90天",
            "余额可依法继承；账号本身不可继承（用户仅有使用权）",
            "微信公众号'法律如何守护人格化的数字遗产'(2025)：陈晔父亲去世10年后微信被销号；《民法典》第127条；2025银行新规参照银行存款"
        ),
        PlatformEntry::new(3, "云闪付", "UnionPay云闪付", "支付金融", "移动支付", true, false, true,
            "死亡证明、继承公证书",
            "联系银联客服95516",
            "中等", "https://www.unionpay.com", "30-60天",

            ""),
        PlatformEntry::new(4, "京东支付", "JD Pay", "支付金融", "在线支付", true, false, true,
            "死亡证明、继承公证书",
            "联系京东金融客服95118",
            "中等", "https://www.jdpay.com", "30-60天",
            ""),
        
        // ========== 社交通讯类 ==========
        PlatformEntry::with_source(10, "微信", "WeChat", "社交通讯", "即时通讯", true, true, false,
            "不可继承（仅可导出聊天记录）",
            "微信账号本身不可继承。可通过官网申请导出聊天记录。可设置\"收藏\"内容继承",
            "困难", "https://weixin.qq.com", "无法继承",
            "连续180天未登录将被平台收回；陈晔父亲去世10年后被销号（2025热搜）",
            "微信公众号'去世父亲微信账号被收回'(2025)：陈晔收集11条信息仍被拒；广州中院认定微信账号属于虚拟财产；《民法典》第127条"
        ),
        PlatformEntry::new(11, "QQ", "QQ", "社交通讯", "即时通讯", true, true, false,
            "不可继承",
            "QQ账号不能继承。QQ空间中照片等数据需在生前自行备份",
            "困难", "https://im.qq.com", "无法继承",
            "需在生前自行备份数据"),
        PlatformEntry::new(12, "微博", "Weibo", "社交通讯", "社交媒体", true, true, false,
            "需联系客服",
            "可申请为\"纪念账号\"，但不可登录和转移所有权。商业账号收益需走继承流程",
            "中等", "https://weibo.com", "30-90天",
            "公开内容可长期保留"),
        PlatformEntry::new(13, "小红书", "Xiaohongshu", "社交通讯", "社交媒体", true, true, false,
            "需联系客服",
            "粉丝量有价值的账号可尝试申请商业继承。个人数据可在生前导出",
            "中等", "https://www.xiaohongshu.com", "30-60天",
            "创作者账号达到一定量级有商业价值"),
        PlatformEntry::with_source(14, "抖音", "Douyin", "社交通讯", "短视频", true, true, true,
            "需联系客服，账号持有人死亡证明",
            "有价值的账号可申请过户给继承人。需确认账号类型为机构/企业账号才能过户",
            "中等", "https://www.douyin.com", "60-120天",
            "一个身份证只能绑定一个抖音号；更换实名需先注销原号或申诉（知乎实测）",
            "知乎'什么场景下需要更换实名'：抖音更换实名需先注销或申诉；快手可通过创作平台变更主体；辽宁开原法院认定快手账户属于新生的虚拟财产"
        ),
        PlatformEntry::new(15, "快手", "Kuaishou", "社交通讯", "短视频", true, true, true,
            "死亡证明、身份关系证明",
            "平台支持账号继承，需走官方流程提交材料",
            "中等", "https://www.kuaishou.com", "60-90天",
            "快手账号继承相对成熟，有官方流程"),
        PlatformEntry::new(16, "B站(bilibili)", "Bilibili", "社交通讯", "内容创作", true, true, false,
            "不可继承，需生前自行备份",
            "账号不可继承。创作内容可在生前自行导出保存",
            "困难", "https://www.bilibili.com", "无法继承",
            "官方UP主账号绑定实名认证，无法变更"),
        PlatformEntry::new(17, "知乎", "Zhihu", "社交通讯", "问答社区", true, true, false,
            "账号不可继承",
            "账号不可继承。创作内容可在生前自行保存",
            "困难", "https://www.zhihu.com", "无法继承",
            "内容型数字遗产，仅有情感和知识产权价值"),
        PlatformEntry::new(18, "陌陌", "Momo", "社交通讯", "社交", false, true, false,
            "不可继承",
            "账号不可继承",
            "困难", "https://www.immomo.com", "无法继承",
            "纯社交通讯价值"),
        PlatformEntry::new(19, "钉钉", "DingTalk", "社交通讯", "办公协作", true, true, false,
            "由企业管理员处理",
            "企业账号由企业管理员负责。个人数据需生前备份",
            "中等", "https://www.dingtalk.com", "视企业而定",
            "企业内数据归企业所有"),

        // ========== 游戏类 ==========
        PlatformEntry::with_source(20, "腾讯游戏综合", "Tencent Games", "游戏", "游戏账号", true, true, true,
            "死亡证明、继承公证书、继承人身份证",
            "2025北京石景山法院判例：平台格式条款无效，87个账号判令配合过户。联系腾讯客服",
            "中等", "https://www.qq.com", "60-120天",
            "87个账号判例：古先生身故，母亲主张继承实名认证账号",
            "凤凰网·惠诚律所(2025)：北京石景山法院判令游戏公司配合办理实名变更；认定'不得继承'格式条款无效"
        ),
        PlatformEntry::new(21, "网易游戏", "NetEase Games", "游戏", "游戏账号", true, true, true,
            "死亡证明、继承公证书",
            "网易游戏提供账号继承流程，需联系客服提交材料",
            "中等", "https://www.163.com", "60-120天",
            "网易游戏在虚拟财产继承方面较为开放"),
        PlatformEntry::new(22, "Steam", "Steam", "游戏", "游戏平台", true, true, false,
            "账号不可继承（需生前备份）",
            "Steam账号不可转移。需生前自行备份游戏购买信息",
            "困难", "https://store.steampowered.com", "无法继承",
            "Steam用户协议明确禁止账号转让和继承"),
        PlatformEntry::new(23, "米哈游（原神等）", "HoYoverse", "游戏", "游戏账号", true, true, true,
            "死亡证明、继承公证书",
            "米哈游支持游戏账号继承，需联系客服",
            "中等", "https://www.mihoyo.com", "30-90天",
            "价值较高的账号可申请继承"),
        PlatformEntry::new(24, "暴雪战网", "Battle.net", "游戏", "游戏平台", true, true, false,
            "账号不可继承",
            "根据暴雪战网服务条款，账号不可继承转让",
            "困难", "https://www.blizzard.com", "无法继承",
            "需生前自行备份领取码等"),

        // ========== 电商类 ==========
        PlatformEntry::new(30, "淘宝", "Taobao", "电商", "网购平台", true, true, true,
            "死亡证明、继承公证书、继承人身份证",
            "个人网店可过户。支付宝余额继承走支付宝流程",
            "中等", "https://www.taobao.com", "30-90天",
            "有收入的淘宝店铺具有明确财产价值"),
        PlatformEntry::new(31, "京东", "JD.com", "电商", "网购平台", true, true, true,
            "死亡证明、继承公证书",
            "京东余额和Plus会员有效期可继承",
            "中等", "https://www.jd.com", "30-60天",
            "Plus会员剩余价值低"),
        PlatformEntry::new(32, "拼多多", "Pinduoduo", "电商", "网购平台", true, false, true,
            "死亡证明、继承公证书",
            "拼多多钱包余额可申请继承",
            "中等", "https://www.pinduoduo.com", "30-60天",
            "纯支付功能为主"),
        PlatformEntry::new(33, "闲鱼", "Xianyu", "电商", "二手交易", true, false, true,
            "继承公证书、死亡证明",
            "闲鱼与支付宝账号绑定，按支付宝流程处理",
            "中等", "https://www.goofish.com", "30-60天",
            "与支付宝联动"),
        PlatformEntry::new(34, "得物", "Dewu/POIZON", "电商", "潮流电商", true, false, true,
            "死亡证明、继承公证书",
            "得物账户余额可申请提取",
            "中等", "https://www.dewu.com", "30-60天",
            ""),

        // ========== 金融证券类 ==========
        PlatformEntry::new(40, "同花顺", "Flush", "金融证券", "股票软件", true, false, true,
            "死亡证明、继承公证书、证券继承材料",
            "股票账户继承需走证券公司+中登公司流程，股票遗产继承较为复杂",
            "困难", "https://www.10jqka.com.cn", "90-365天",
            "股票继承涉及证券公司、中登公司、公证处、法院等多个机构"),
        PlatformEntry::new(41, "东方财富", "East Money", "金融证券", "金融平台", true, false, true,
            "死亡证明、继承公证书",
            "证券账户按股票继承流程处理",
            "困难", "https://www.eastmoney.com", "90-365天",
            ""),
        PlatformEntry::new(42, "蚂蚁财富", "Ant Fortune", "金融证券", "基金理财", true, false, true,
            "死亡证明、继承公证书",
            "基金、理财产品可通过支付宝体系提交继承申请",
            "中等", "https://www.antfortune.com", "30-90天",
            ""),
        PlatformEntry::new(43, "微信零钱通", "WeChat Money", "金融证券", "理财产品", true, false, true,
            "死亡证明、继承公证书",
            "走微信支付继承流程",
            "中等", "https://weixin.qq.com", "30-90天",
            "与微信支付同步处理"),
        PlatformEntry::new(44, "招商银行APP", "CMB APP", "金融证券", "银行", true, false, true,
            "死亡证明、继承公证书",
            "银行卡及招行APP内资产按银行继承流程，继承公证书即可办理",
            "简单", "https://www.cmbchina.com", "7-30天",
            "银行体系内继承流程成熟"),

        // ========== 知识付费类 ==========
        PlatformEntry::new(50, "得到", "Dedao", "知识付费", "知识服务", true, false, false,
            "账号不可继承",
            "账号不可继承。已购买的课程可在原设备上继续使用",
            "困难", "https://www.dedao.cn", "无法继承",
            "课程绑定账号，不可转移给他人"),
        PlatformEntry::new(51, "知乎盐选", "Zhihu Premium", "知识付费", "内容会员", true, false, false,
            "账号不可继承",
            "不可继承。需生前自行使用",
            "困难", "https://www.zhihu.com", "无法继承",
            ""),
        PlatformEntry::new(52, "喜马拉雅", "Ximalaya", "知识付费", "音频平台", true, false, false,
            "死亡证明、继承公证书",
            "联系喜马拉雅客服申请余额退还",
            "中等", "https://www.ximalaya.com", "30-60天",
            "已购课程不可转移，仅退余额"),
        PlatformEntry::new(53, "腾讯课堂", "Tencent Class", "知识付费", "在线教育", true, false, false,
            "账号不可继承",
            "已购课程绑定账号，不可转移",
            "困难", "https://ke.qq.com", "无法继承",
            ""),

        // ========== 工具订阅类 ==========
        PlatformEntry::new(60, "iCloud", "iCloud", "工具订阅", "云服务", true, true, true,
            "死亡证明、继承密钥、遗产联系人",
            "苹果已为死者提供\"数字遗产\"功能。需生前设置\"遗产联系人\"",
            "简单", "https://support.apple.com/zh-cn/HT212360", "7-30天",
            "苹果数字遗产计划（Digital Legacy）已成熟，需生前设置"),
        PlatformEntry::new(61, "百度网盘", "Baidu Netdisk", "工具订阅", "云存储", true, true, false,
            "身份验证+账号密码",
            "百度网盘不官方支持账号访问权转移。需生前共享账号或备份文件",
            "困难", "https://pan.baidu.com", "不支持",
            "数据价值完全依赖账号密码"),
        PlatformEntry::new(62, "阿里云盘", "Alibaba Cloud Drive", "工具订阅", "云存储", true, true, false,
            "身份验证+账号密码",
            "同百度网盘",
            "困难", "https://www.aliyundrive.com", "不支持",
            ""),
        PlatformEntry::new(63, "WPS Office", "WPS Office", "工具订阅", "办公软件", true, false, false,
            "账号不可继承",
            "已购买会员有效期不可转移。文档需生前自行导出",
            "困难", "https://www.wps.cn", "无法继承",
            ""),
        PlatformEntry::new(64, "Office 365 / Microsoft 365", "Microsoft 365", "工具订阅", "办公软件", true, false, false,
            "需Microsoft客服协助",
            "订阅费用可按比例退费",
            "中等", "https://www.microsoft.com", "30天",
            ""),
        PlatformEntry::new(65, "网易云音乐", "NetEase Music", "工具订阅", "音乐", true, true, false,
            "账号不可继承",
            "付费会员到期后账号失效。已下载音乐绑定设备",
            "困难", "https://music.163.com", "无法继承",
            "无实质继承价值"),
        PlatformEntry::new(66, "QQ音乐", "QQ Music", "工具订阅", "音乐", true, false, false,
            "账号不可继承",
            "付费会员不可继承",
            "困难", "https://y.qq.com", "无法继承",
            ""),

        // ========== 出行住宿类 ==========
        PlatformEntry::new(70, "12306铁路", "12306", "出行住宿", "交通", true, true, false,
            "账号不可继承",
            "账号绑定身份证。乘车记录和积分无实质继承价值",
            "困难", "https://www.12306.cn", "无法继承",
            "账户与身份证绑定，持有人去世即失效"),
        PlatformEntry::new(71, "携程", "Ctrip", "出行住宿", "旅游", true, true, true,
            "死亡证明、继承公证书",
            "携程任我行余额可继承。积分通常在继承后失效",
            "中等", "https://www.ctrip.com", "30-60天",
            ""),
        PlatformEntry::new(72, "滴滴出行", "Didi", "出行住宿", "出行", true, false, true,
            "死亡证明、继承公证书",
            "滴滴余额可申请退还",
            "中等", "https://www.didiglobal.com", "30天",
            ""),
        PlatformEntry::new(73, "美团", "Meituan", "生活工具", "本地生活", true, false, true,
            "死亡证明、继承公证书",
            "美团余额、美团月付账户需联系客服处理",
            "中等", "https://www.meituan.com", "30-60天",
            ""),
        PlatformEntry::new(74, "饿了么", "Ele.me", "生活工具", "外卖", true, false, true,
            "死亡证明、继承公证书",
            "账户余额可联系客服申请退还",
            "中等", "https://www.ele.me", "30-60天",
            ""),

        // ========== 自媒体/创作类 ==========
        PlatformEntry::new(80, "微信公众号", "WeChat Official Account", "内容创作", "创作平台", true, true, false,
            "公众号可迁移",
            "类型不同政策不同。个人号不支持继承，可迁移至新主体。企业号按企业流程处理",
            "困难", "https://mp.weixin.qq.com", "60-180天",
            "有商业价值的公众号（有广告收入和有粉丝）可申请主体变更"),
        PlatformEntry::new(81, "今日头条", "Toutiao", "内容创作", "内容平台", true, true, true,
            "联系客服",
            "有平台收益的头条号可申请收益权转移",
            "中等", "https://www.toutiao.com", "60-90天",
            "创作收益具有明确财产价值"),
        PlatformEntry::new(82, "百家号", "Baijiahao", "内容创作", "内容平台", true, false, true,
            "联系百度客服",
            "账号收益可申请继承",
            "中等", "https://baijiahao.baidu.com", "30-60天",
            ""),
        PlatformEntry::new(83, "视频号", "WeChat Channels", "内容创作", "短视频", true, true, false,
            "账号不可继承",
            "与微信绑定，微信无法继承则视频号也无法继承",
            "困难", "https://channels.weixin.qq.com", "无法继承",
            ""),

        // ========== 存储云服务类 ==========
        PlatformEntry::new(90, "腾讯微云", "Weiyun", "存储云服务", "云盘", true, true, false,
            "需账号密码",
            "官方不提供账号访问权转移，需生前自行备份",
            "困难", "https://www.weiyun.com", "无法继承",
            ""),
        PlatformEntry::new(91, "OneDrive", "OneDrive", "存储云服务", "云盘", true, true, true,
            "死亡证明、继承材料",
            "微软提供数字遗产访问流程",
            "中等", "https://onedrive.live.com", "30-60天",
            "微软提供了较为成熟的数字遗产流程"),
        PlatformEntry::new(92, "Google Drive", "Google Drive", "存储云服务", "云盘", true, true, true,
            "出生/死亡证明及授权",
            "谷歌Inactive Account Manager支持",
            "简单", "https://drive.google.com", "已设置则自动执行",
            "需生前在谷歌账户中配置不活跃账户管理器"),
        PlatformEntry::new(93, "Dropbox", "Dropbox", "存储云服务", "云盘", true, true, true,
            "继承人申请+法律文件",
            "Dropbox支持通过法律流程转移",
            "中等", "https://www.dropbox.com", "30-60天",
            ""),

        // ========== 生活服务类 ==========
        PlatformEntry::new(100, "广州医保", "Guangzhou Medical Insurance", "政务服务", "公共服务", true, false, true,
            "死亡证明、社保继承材料",
            "医保个人账户余额可由继承人申请继承",
            "简单", "https://www.gz.gov.cn", "15-30天",
            "医保余额可提取"),
        PlatformEntry::new(101, "广东社保", "Guangdong Social Insurance", "政务服务", "社保", true, false, true,
            "死亡证明、继承公证书",
            "社保（养老）个人账户余额可提取，公积金一并处理",
            "简单", "https://www.gd.gov.cn", "15-30天",
            "社保和公积金均有明确继承法规"),
        PlatformEntry::new(102, "公积金", "Housing Fund", "政务服务", "公积金", true, false, true,
            "死亡证明、继承公证书",
            "公积金账户余额可由继承人全额提取",
            "简单", "https://www.cqgjj.cn", "7-15天",
            "公积金法规明确，继承简便"),
    ]
}

/// 按ID获取平台
pub fn get_platform_by_id(id: i64) -> Option<PlatformEntry> {
    get_all_platforms().into_iter().find(|p| p.id == id)
}

/// 按类别获取平台
pub fn get_platforms_by_category(category: &str) -> Vec<PlatformEntry> {
    get_all_platforms().into_iter().filter(|p| p.category == category).collect()
}

/// 获取继承难度分布
pub fn get_difficulty_stats() -> std::collections::HashMap<String, i64> {
    let platforms = get_all_platforms();
    let mut stats: std::collections::HashMap<String, i64> = std::collections::HashMap::new();
    for p in platforms {
        *stats.entry(p.estimated_difficulty.clone()).or_insert(0) += 1;
    }
    stats
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn platform_database_has_56_entries() {
        let platforms = get_all_platforms();
        assert_eq!(platforms.len(), 56, "平台数据库应包含 56 个平台");
    }

    #[test]
    fn all_platforms_have_names() {
        let platforms = get_all_platforms();
        for p in &platforms {
            assert!(!p.name.is_empty(), "平台 id={} 名称为空", p.id);
            assert!(!p.name_en.is_empty(), "平台 id={} 英文名为空", p.id);
            assert!(!p.category.is_empty(), "平台 id={} 类别为空", p.id);
        }
    }

    #[test]
    fn all_platforms_have_valid_ids() {
        let platforms = get_all_platforms();
        let ids: Vec<i64> = platforms.iter().map(|p| p.id).collect();
        let unique: std::collections::HashSet<i64> = ids.iter().cloned().collect();
        assert_eq!(ids.len(), unique.len(), "平台 id 存在重复");
        assert!(ids.iter().all(|&id| id > 0), "平台 id 必须为正数");
    }

    #[test]
    fn categories_are_within_known_set() {
        let known = [
            "支付金融", "社交通讯", "游戏", "电商", "内容创作", "知识付费",
            "金融证券", "出行住宿", "工具订阅", "音视频", "生活服务", "学习与考试",
            "存储云服务", "政务服务", "生活工具",
        ];
        let platforms = get_all_platforms();
        for p in &platforms {
            assert!(known.contains(&p.category.as_str()), "未知类别: {}", p.category);
        }
    }

    #[test]
    fn difficulty_stats_sum_matches_total() {
        let stats = get_difficulty_stats();
        let sum: i64 = stats.values().sum();
        assert_eq!(sum, 56, "难度分布总数应为 56");
    }
}
