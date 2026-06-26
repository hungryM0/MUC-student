---
name: muc-student-network-login-flow
description: 梳理或修改 MUC-student 的校园网认证、轻量 portal 切号、成功页 SSO 免密自助面板查询、流量查询、本机下线、在线设备识别和自动切号链路。仅在改 `src-core/src/application/backend.rs`、`src-core/src/application/services/*traffic*`、`src-core/src/application/services/portal_snapshot_service.rs`、`src-core/src/infrastructure/network/*portal*`、`src-core/src/infrastructure/network/self_service_panel_client.rs` 或相关 parser/策略时使用；普通前端改动不触发。
---

# MUC Student Network Login Flow

先找入口。不要先改 parser，也不要先改某个 HTTP client。

## 入口顺序

先从 `src-core/src/application/backend.rs` 开始读，再沿着这条线往下：

- `login_selected_account` / `login_selected_account_inner`
- `refresh_dashboard` / `run_refresh` / `refresh_inner`
- `logout_local_device` / `logout_local_device_inner`
- `try_auto_switch`

再看它们调到哪里：

- `LegacyPortalAuthClient`
- `LegacyPortalStatusClient`
- `SelfServicePanelClient`
- `AccountTrafficService`
- `portal_snapshot_service`
- `NetworkStatusService`
- `traffic_math`
- 各 parser

## 读代码顺序

1. 先确认改的是哪条链路。
2. 找 application 层入口。
3. 记下 DTO、日志、运行时状态会不会受影响。
4. 再往下读 infrastructure client 和 parser。
5. 最后才改策略和计算逻辑。

## 当前关键链路

### 登录

- 入口在 `Backend::login_selected_account`
- 实际流程在 `login_selected_account_inner`
- 会先探测本机网络状态
- 可能先从在线列表里找当前本机 IP 所属账号
- 切号不要先本机下线
- 切号应走 `http://rz.muc.edu.cn/srun_portal_pc.php?ac_id=1&` 登录页表单覆盖登录
- 登录页表单 POST 到 `srun_portal_pc.php`，字段包含 `action=login`、`username`、明文 `password`、`ac_id=1`、`save_me=0`、`ajax=1`
- `/include/auth_action.php` 的轻量接口可能返回 `IP has been online, please logout.`，不能把它当切号主链路
- 最后写回 `app_state`、选中账号、日志和事件

### 刷新状态

- 入口在 `run_refresh` / `refresh_inner`
- 先查本机 IP
- 优先走轻量 SSO 免密查询：`LegacyPortalStatusClient::fetch_success_info` 读 `srun_portal_pc_success.php`
- 成功页必须能解析出本机 IP、上网用户、已用流量、计费方式
- 用成功页账号匹配本地账号后，`SelfServicePanelClient::fetch_sso_html` 访问自助服务 `/home`
- SSO URL 规则是 `traffic_portal_url` 的 origin + `/site/sso?data=base64(username:username)`
- SSO 第一跳 302 到 `/home`，必须保留这一跳返回的 `PHPSESSID_8800` cookie，再请求 `/home`
- `/home` 里解析产品信息、计费策略、已用流量、在线设备、下线链接和套餐相关文本
- SSO 或 `/home` 失败时，回退 `portal_snapshot_service::build_single_success_snapshot`，只用成功页数据和缓存补齐
- 不要在刷新里串行扫描所有账号；刷新只查当前本机在线账号
- 用 `build_status_card_order`、`save_cached_traffic_snapshots` 写回缓存
- 刷新后可能触发 `try_auto_switch`

### 轻量 SSO 免密查询

关键事实：

- 成功页是 `http://rz.muc.edu.cn/srun_portal_pc_success.php`
- 页面里的“自助服务”按钮形如 `http://192.168.2.231:8800/site/sso?data=...`
- `data` 不是随机 token，而是 `base64(username:username)`
- 例子：`25011777:25011777` -> `MjUwMTE3Nzc6MjUwMTE3Nzc=`
- 这条链路不需要账号密码，不需要 OCR，也不需要登录自助面板表单
- 自动化时不能简单 `curl -L` 丢 cookie；要保留 `/site/sso` 302 设置的 cookie，再请求 `/home`

代码落点：

- `LegacyPortalStatusClient::fetch_success_info` 取成功页
- `legacy_portal_success_page_parser` 解析当前 IP、上网用户、已用流量、计费方式
- `SelfServicePanelClient::fetch_sso_html` / `fetch_sso_page` 走免密 SSO
- `AccountTrafficService::snapshot_from_panel_home` 把 `/home` HTML 转成 `AccountTrafficSnapshot`
- `panel_home_parser` 解析产品表和在线设备
- `online_device_parser` 解析 `/home/delete` 下线链接

不要把这条链路改回“带密码查所有账号”。那是慢路径，容易把所有账号都打到面板，屎山味很冲。

### 本机下线

- 入口在 `logout_local_device_inner`
- 必须先有有效本机 IP
- 必须先有 `current_online_account_id`
- 实际 portal 下线由 `LegacyPortalAuthClient::logout_current_ip` 完成
- 本机下线只用于用户主动下号，不用于切号前置步骤

## 硬规则

- 不要跳过 application 层，直接让前端猜 infrastructure 细节。
- 不要把 parser 当业务层。parser 只做解析，不做流程判断。
- 自动切号规则放 `domain/policies`，不要塞进 HTTP client。
- 修改登录或刷新行为时，同时检查日志、状态事件、缓存写回。

## 常见坑

- 只改 `LegacyPortalAuthClient`，忘了 `PortalSnapshotService` 的并发探测和串行回退。
- 只改 response 解析，忘了 `already_online` 和预下线分支。
- 把切号做成“先下线再登录”。这是错的，登录页本身支持覆盖登录。
- 只改后端，不看前端是否依赖 `AppSnapshotDto` 字段。
- 在 infrastructure 层写选择账号逻辑，代码会发臭。
- 又把 OCR 或验证码登录链路加回来，直接判定越界。

## 参考

- 调用链索引：`references/call-graph.md`
- 关键模块职责：`references/modules.md`
