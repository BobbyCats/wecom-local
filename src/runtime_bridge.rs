use anyhow::{bail, Context, Result};
use std::path::{Path, PathBuf};

#[derive(Debug, Clone)]
pub struct WeComProcess {
    pub pid: u32,
    pub name: String,
}

#[cfg(target_os = "macos")]
pub fn export_history(
    conversation_id: &str,
    limit: usize,
    offset: usize,
) -> Result<serde_json::Value> {
    let target_output = wecom_container_tmp()?.join(unique_name("wecom-local-runtime", "json"));
    let expr = build_lldb_expression(conversation_id, limit, offset, &target_output);
    run_lldb_expression(&expr, &target_output)
}

#[cfg(target_os = "macos")]
pub fn list_conversations() -> Result<serde_json::Value> {
    let target_output =
        wecom_container_tmp()?.join(unique_name("wecom-local-conversations", "json"));
    let expr = build_conversation_list_expression(&target_output);
    run_lldb_expression(&expr, &target_output)
}

#[cfg(target_os = "macos")]
pub fn list_members(conversation_id: &str) -> Result<serde_json::Value> {
    let target_output = wecom_container_tmp()?.join(unique_name("wecom-local-members", "json"));
    let expr = build_members_expression(conversation_id, &target_output);
    run_lldb_expression(&expr, &target_output)
}

#[cfg(target_os = "macos")]
pub fn export_history_and_members(
    conversation_id: &str,
    limit: usize,
    offset: usize,
) -> Result<(serde_json::Value, serde_json::Value)> {
    let output_dir = wecom_container_tmp()?;
    let history_output = output_dir.join(unique_name("wecom-local-runtime", "json"));
    let members_output = output_dir.join(unique_name("wecom-local-members", "json"));
    let history_expr = build_lldb_expression(conversation_id, limit, offset, &history_output);
    let members_expr = build_members_expression(conversation_id, &members_output);
    let mut values = run_lldb_expressions(&[
        (history_expr.as_str(), history_output.as_path()),
        (members_expr.as_str(), members_output.as_path()),
    ])?;
    let members = values.pop().context("missing member payload")?;
    let history = values.pop().context("missing history payload")?;
    Ok((history, members))
}

#[cfg(target_os = "macos")]
fn run_lldb_expression(expr: &str, target_output: &Path) -> Result<serde_json::Value> {
    let mut values = run_lldb_expressions(&[(expr, target_output)])?;
    values.pop().context("missing runtime payload")
}

#[cfg(target_os = "macos")]
fn run_lldb_expressions(expressions: &[(&str, &Path)]) -> Result<Vec<serde_json::Value>> {
    use std::fs;
    use std::process::Command;

    let process = find_wecom_process()?;
    let script_path = std::env::temp_dir().join(unique_name("wecom-local-lldb", "lldb"));
    let script = build_lldb_script(expressions.iter().map(|(expr, _)| *expr));

    fs::write(&script_path, script)
        .with_context(|| format!("failed to write LLDB script: {}", script_path.display()))?;

    let output = Command::new("/usr/bin/lldb")
        .arg("-b")
        .arg("-p")
        .arg(process.pid.to_string())
        .arg("-s")
        .arg(&script_path)
        .output()
        .context("failed to start LLDB")?;

    let _ = fs::remove_file(&script_path);

    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);
    if !output.status.success() {
        bail!(
            "failed to read WeCom runtime. Try running this command with sudo.{}",
            lldb_failure_detail(&stdout, &stderr)
        );
    }

    let mut values = Vec::with_capacity(expressions.len());
    for (_, target_output) in expressions {
        if !target_output.exists() {
            bail!(
                "LLDB finished but did not create the runtime export file. The WeCom runtime interface may have changed.{}",
                lldb_failure_detail(&stdout, &stderr)
            );
        }

        let raw = fs::read_to_string(target_output).with_context(|| {
            format!("failed to read runtime export: {}", target_output.display())
        })?;
        let _ = fs::remove_file(target_output);
        values.push(serde_json::from_str(&raw).context("failed to parse WeCom runtime JSON")?);
    }

    Ok(values)
}

#[cfg(target_os = "macos")]
fn build_lldb_script<'a>(expressions: impl IntoIterator<Item = &'a str>) -> String {
    let mut script = String::new();
    for expr in expressions {
        script.push_str("expr -l objc++ -O -- ");
        script.push_str(&one_line(expr));
        script.push('\n');
    }
    script.push_str("process detach\nquit\n");
    script
}

#[cfg(target_os = "macos")]
fn lldb_failure_detail(stdout: &str, stderr: &str) -> String {
    if std::env::var("WECOM_LOCAL_DEBUG_LLDB").as_deref() != Ok("1") {
        return " Set WECOM_LOCAL_DEBUG_LLDB=1 to include sanitized LLDB diagnostics.".to_string();
    }

    let detail = format!("{}{}", stdout, stderr);
    let sanitized = redact_runtime_debug_output(&detail);
    if sanitized.trim().is_empty() {
        String::new()
    } else {
        format!("\n{}", sanitized)
    }
}

#[cfg(target_os = "macos")]
fn redact_runtime_debug_output(value: &str) -> String {
    value
        .lines()
        .filter(|line| !line.starts_with("(lldb) expr "))
        .map(redact_runtime_debug_line)
        .collect::<Vec<_>>()
        .join("\n")
}

#[cfg(target_os = "macos")]
fn redact_runtime_debug_line(line: &str) -> String {
    line.split_whitespace()
        .map(|token| {
            if token.starts_with("R:") || token.starts_with("S:") {
                "<redacted-conversation-id>".to_string()
            } else {
                token.to_string()
            }
        })
        .collect::<Vec<_>>()
        .join(" ")
}

#[cfg(not(target_os = "macos"))]
pub fn export_history(
    _conversation_id: &str,
    _limit: usize,
    _offset: usize,
) -> Result<serde_json::Value> {
    bail!("WeCom runtime history export currently supports macOS only")
}

#[cfg(not(target_os = "macos"))]
pub fn list_conversations() -> Result<serde_json::Value> {
    bail!("WeCom runtime conversation discovery currently supports macOS only")
}

#[cfg(not(target_os = "macos"))]
pub fn list_members(_conversation_id: &str) -> Result<serde_json::Value> {
    bail!("WeCom runtime member discovery currently supports macOS only")
}

#[cfg(not(target_os = "macos"))]
pub fn export_history_and_members(
    _conversation_id: &str,
    _limit: usize,
    _offset: usize,
) -> Result<(serde_json::Value, serde_json::Value)> {
    bail!("WeCom runtime history and member discovery currently supports macOS only")
}

#[cfg(target_os = "macos")]
pub fn find_wecom_process() -> Result<WeComProcess> {
    use std::process::Command;

    for name in ["企业微信", "WeWork"] {
        let output = Command::new("pgrep")
            .arg("-x")
            .arg(name)
            .output()
            .context("failed to run pgrep")?;
        if output.status.success() {
            let text = String::from_utf8_lossy(&output.stdout);
            if let Some(pid) = text
                .lines()
                .find_map(|line| line.trim().parse::<u32>().ok())
            {
                return Ok(WeComProcess {
                    pid,
                    name: name.to_string(),
                });
            }
        }
    }
    bail!("WeCom Desktop is not running")
}

#[cfg(not(target_os = "macos"))]
pub fn find_wecom_process() -> Result<WeComProcess> {
    bail!("WeCom process discovery currently supports macOS only")
}

#[cfg(target_os = "macos")]
pub fn wecom_container_tmp() -> Result<PathBuf> {
    let path = invoking_user_home().join("Library/Containers/com.tencent.WeWorkMac/Data/tmp");
    std::fs::create_dir_all(&path)
        .with_context(|| format!("failed to create WeCom container tmp: {}", path.display()))?;
    Ok(path)
}

#[cfg(not(target_os = "macos"))]
pub fn wecom_container_tmp() -> Result<PathBuf> {
    bail!("WeCom container tmp discovery currently supports macOS only")
}

#[cfg(target_os = "macos")]
fn invoking_user_home() -> PathBuf {
    #[cfg(unix)]
    {
        use std::ffi::{CStr, CString};
        if let Ok(sudo_user) = std::env::var("SUDO_USER") {
            if let Ok(c_user) = CString::new(sudo_user) {
                unsafe {
                    let pwd = libc::getpwnam(c_user.as_ptr());
                    if !pwd.is_null() && !(*pwd).pw_dir.is_null() {
                        if let Ok(path) = CStr::from_ptr((*pwd).pw_dir).to_str() {
                            return PathBuf::from(path);
                        }
                    }
                }
            }
        }
    }
    dirs::home_dir().unwrap_or_else(|| PathBuf::from("/tmp"))
}

#[cfg(target_os = "macos")]
fn unique_name(prefix: &str, ext: &str) -> String {
    let nanos = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_nanos();
    format!("{}-{}-{}.{}", prefix, std::process::id(), nanos, ext)
}

#[cfg(target_os = "macos")]
fn objc_string_literal(value: &str) -> String {
    let escaped = value
        .replace('\\', "\\\\")
        .replace('"', "\\\"")
        .replace('\n', "\\n")
        .replace('\r', "\\r");
    format!("@\"{}\"", escaped)
}

#[cfg(target_os = "macos")]
fn one_line(value: &str) -> String {
    value
        .lines()
        .map(str::trim)
        .filter(|line| !line.is_empty())
        .collect::<Vec<_>>()
        .join(" ")
}

#[cfg(target_os = "macos")]
fn build_conversation_list_expression(output_path: &Path) -> String {
    let out = objc_string_literal(&output_path.to_string_lossy());
    format!(
        r#"
@import Foundation; @import ObjectiveC;
id (*__whObj)(id, SEL) = (id (*)(id, SEL))objc_msgSend;
id (*__whObj1)(id, SEL, id) = (id (*)(id, SEL, id))objc_msgSend;
unsigned long long (*__whUll)(id, SEL) = (unsigned long long (*)(id, SEL))objc_msgSend;
long long (*__whLl)(id, SEL) = (long long (*)(id, SEL))objc_msgSend;
double (*__whDbl)(id, SEL) = (double (*)(id, SEL))objc_msgSend;
BOOL (*__whBool)(id, SEL) = (BOOL (*)(id, SEL))objc_msgSend;
BOOL (*__whResponds)(id, SEL, SEL) = (BOOL (*)(id, SEL, SEL))objc_msgSend;
NSString *__whOutputPath = {out};
id __whMgr = [(id)objc_getClass("WEWServiceManager") defaultServiceManager];
id __whConversationSvc = [__whMgr conversationService];
NSMutableDictionary *__whPayload = [NSMutableDictionary dictionary];
NSMutableArray *__whRows = [NSMutableArray array];
[__whPayload setObject:__whRows forKey:@"conversations"];
SEL __whListSel = sel_registerName("getAllActiveAndUnblockedConversationIds");
SEL __whConvSel = sel_registerName("conversationWithId:");
SEL __whRespondsSel = sel_registerName("respondsToSelector:");
if (!__whConversationSvc || !__whResponds(__whConversationSvc, __whRespondsSel, __whListSel) || !__whResponds(__whConversationSvc, __whRespondsSel, __whConvSel)) {{
    [__whPayload setObject:@"conversation list selector unavailable" forKey:@"error"];
    [__whPayload setObject:@0 forKey:@"total_count"];
    [__whPayload setObject:@0 forKey:@"matched_count"];
}} else {{
    NSArray *__whIds = __whObj(__whConversationSvc, __whListSel) ?: @[];
    NSDateFormatter *__whDateFormatter = [[NSDateFormatter alloc] init];
    [__whDateFormatter setDateFormat:@"yyyy-MM-dd HH:mm:ss"];
    for (id __whConversationIdValue in __whIds) {{
        NSString *__whConversationId = (NSString *)__whConversationIdValue;
        id __whConversation = __whObj1(__whConversationSvc, __whConvSel, __whConversationId);
        if (!__whConversation) {{ continue; }}
        NSString *__whName = __whObj(__whConversation, sel_registerName("name")) ?: @"";
        NSDate *__whModifyTime = __whObj(__whConversation, sel_registerName("modifyTime"));
        NSDate *__whCreateTime = __whObj(__whConversation, sel_registerName("createTime"));
        NSString *__whModifyTimeText = __whModifyTime ? [__whDateFormatter stringFromDate:__whModifyTime] : @"";
        NSString *__whCreateTimeText = __whCreateTime ? [__whDateFormatter stringFromDate:__whCreateTime] : @"";
        [__whRows addObject:@{{
            @"conversation_id": __whConversationId ?: @"",
            @"conversation_name": __whName ?: @"",
            @"conversation_type": @(__whLl(__whConversation, sel_registerName("conversationType"))),
            @"last_message_id": @(__whUll(__whConversation, sel_registerName("lastMessageId"))),
            @"modify_time": __whModifyTime ? @(__whDbl(__whModifyTime, sel_registerName("timeIntervalSince1970"))) : @0,
            @"modify_time_text": __whModifyTimeText,
            @"create_time": __whCreateTime ? @(__whDbl(__whCreateTime, sel_registerName("timeIntervalSince1970"))) : @0,
            @"create_time_text": __whCreateTimeText,
            @"is_sticked": @(__whBool(__whConversation, sel_registerName("isSticked"))),
            @"is_marked": @(__whBool(__whConversation, sel_registerName("isMarked"))),
            @"is_blocked": @(__whBool(__whConversation, sel_registerName("isBlocked")))
        }}];
    }}
    [__whPayload setObject:@([__whIds count]) forKey:@"total_count"];
    [__whPayload setObject:@([__whRows count]) forKey:@"matched_count"];
}}
NSError *__whJsonError = nil;
NSData *__whJson = [NSJSONSerialization dataWithJSONObject:__whPayload options:NSJSONWritingPrettyPrinted error:&__whJsonError];
if (__whJson) {{ [__whJson writeToFile:__whOutputPath atomically:YES]; }}
"#
    )
}

#[cfg(target_os = "macos")]
fn build_lldb_expression(
    conversation_id: &str,
    limit: usize,
    offset: usize,
    output_path: &Path,
) -> String {
    let conv = objc_string_literal(conversation_id);
    let out = objc_string_literal(&output_path.to_string_lossy());
    format!(
        r#"
@import Foundation; @import ObjectiveC;
id (*__whObj)(id, SEL) = (id (*)(id, SEL))objc_msgSend;
unsigned long long (*__whUll)(id, SEL) = (unsigned long long (*)(id, SEL))objc_msgSend;
unsigned int (*__whUInt)(id, SEL) = (unsigned int (*)(id, SEL))objc_msgSend;
long long (*__whLl)(id, SEL) = (long long (*)(id, SEL))objc_msgSend;
double (*__whDbl)(id, SEL) = (double (*)(id, SEL))objc_msgSend;
NSString *__whConversationId = {conv};
NSString *__whOutputPath = {out};
id __whMgr = [(id)objc_getClass("WEWServiceManager") defaultServiceManager];
id __whMessageSvc = [__whMgr messageService];
id __whConversationSvc = [__whMgr conversationService];
id __whConversation = [__whConversationSvc conversationWithId:__whConversationId];
NSArray *__whIds = [__whMessageSvc getConversationMessageIds:__whConversationId];
NSUInteger __whTotal = [__whIds count];
NSUInteger __whOffset = (NSUInteger){offset};
NSUInteger __whLimit = (NSUInteger){limit};
NSUInteger __whStart = __whOffset < __whTotal ? __whOffset : __whTotal;
NSUInteger __whAvail = __whTotal - __whStart;
NSUInteger __whTake = (__whLimit == 0 || __whLimit > __whAvail) ? __whAvail : __whLimit;
NSArray *__whSlice = [__whIds subarrayWithRange:NSMakeRange(__whStart, __whTake)];
NSArray *__whMessages = [__whMessageSvc messagesByMessageIds:__whSlice];
NSMutableArray *__whRows = [NSMutableArray arrayWithCapacity:[__whMessages count]];
for (id __whMessage in __whMessages) {{
    NSData *__whContent = __whObj(__whMessage, sel_registerName("content"));
    NSData *__whRawContent = __whObj(__whMessage, sel_registerName("rawContent"));
    NSDate *__whSendTime = __whObj(__whMessage, sel_registerName("sendTime"));
    NSString *__whContentB64 = __whContent ? [__whContent base64EncodedStringWithOptions:0] : @"";
    NSString *__whRawContentB64 = __whRawContent ? [__whRawContent base64EncodedStringWithOptions:0] : @"";
    id __whName = __whObj(__whMessage, sel_registerName("name"));
    id __whNormalUserName = __whObj(__whMessage, sel_registerName("normalUserName"));
    id __whSenderName = __whObj(__whMessage, sel_registerName("senderName"));
    id __whSenderEnglishName = __whObj(__whMessage, sel_registerName("senderEnglishName"));
    id __whRealSenderName = __whObj(__whMessage, sel_registerName("realSenderName"));
    id __whWxNickName = __whObj(__whMessage, sel_registerName("wxNickName"));
    id __whSummary = __whObj(__whMessage, sel_registerName("summaryContent"));
    id __whSummaryTips = __whObj(__whMessage, sel_registerName("summaryTips"));
    [__whRows addObject:@{{
        @"message_id": @(__whUll(__whMessage, sel_registerName("messageId"))),
        @"server_id": @(__whUll(__whMessage, sel_registerName("serverId"))),
        @"seq": @(__whUInt(__whMessage, sel_registerName("seq"))),
        @"sender_id": @(__whLl(__whMessage, sel_registerName("senderId"))),
        @"sender_name": __whSenderName ?: @"",
        @"sender_english_name": __whSenderEnglishName ?: @"",
        @"real_sender_name": __whRealSenderName ?: @"",
        @"wx_nick_name": __whWxNickName ?: @"",
        @"name": __whName ?: @"",
        @"normal_user_name": __whNormalUserName ?: @"",
        @"conversation_id": __whObj(__whMessage, sel_registerName("conversationId")) ?: @"",
        @"content_type": @(__whLl(__whMessage, sel_registerName("contentType"))),
        @"send_time": @(__whDbl(__whSendTime, sel_registerName("timeIntervalSince1970"))),
        @"is_read": @((BOOL)__whLl(__whMessage, sel_registerName("isRead"))),
        @"is_revoke": @((BOOL)__whLl(__whMessage, sel_registerName("isRevoke"))),
        @"has_quote_message": @((BOOL)__whLl(__whMessage, sel_registerName("hasQuoteMessage"))),
        @"summary_content": __whSummary ?: @"",
        @"summary_tips": __whSummaryTips ?: @"",
        @"content_base64": __whContentB64,
        @"raw_content_base64": __whRawContentB64
    }}];
}}
NSMutableDictionary *__whPayload = [NSMutableDictionary dictionary];
[__whPayload setObject:__whConversationId forKey:@"conversation_id"];
[__whPayload setObject:@(__whTotal) forKey:@"total_message_ids"];
[__whPayload setObject:@(__whOffset) forKey:@"offset"];
[__whPayload setObject:@(__whTake) forKey:@"exported_count"];
[__whPayload setObject:__whConversation ? (__whObj(__whConversation, sel_registerName("name")) ?: @"") : @"" forKey:@"conversation_name"];
[__whPayload setObject:__whConversation ? @(__whUll(__whConversation, sel_registerName("lastMessageId"))) : @0 forKey:@"conversation_last_message_id"];
[__whPayload setObject:__whRows forKey:@"messages"];
NSError *__whJsonError = nil;
NSData *__whJson = [NSJSONSerialization dataWithJSONObject:__whPayload options:NSJSONWritingPrettyPrinted error:&__whJsonError];
if (__whJson) {{ [__whJson writeToFile:__whOutputPath atomically:YES]; }}
"#
    )
}

#[cfg(target_os = "macos")]
fn build_members_expression(conversation_id: &str, output_path: &Path) -> String {
    let conv = objc_string_literal(conversation_id);
    let out = objc_string_literal(&output_path.to_string_lossy());
    format!(
        r#"
@import Foundation; @import ObjectiveC;
id (*__wmObj)(id, SEL) = (id (*)(id, SEL))objc_msgSend;
id (*__wmObj1)(id, SEL, id) = (id (*)(id, SEL, id))objc_msgSend;
void (*__wmVoid1)(id, SEL, id) = (void (*)(id, SEL, id))objc_msgSend;
BOOL (*__wmResponds)(id, SEL, SEL) = (BOOL (*)(id, SEL, SEL))objc_msgSend;
BOOL (*__wmBool)(id, SEL) = (BOOL (*)(id, SEL))objc_msgSend;
BOOL (*__wmIsKind)(id, SEL, Class) = (BOOL (*)(id, SEL, Class))objc_msgSend;
long long (*__wmLl)(id, SEL) = (long long (*)(id, SEL))objc_msgSend;
unsigned long long (*__wmUll)(id, SEL) = (unsigned long long (*)(id, SEL))objc_msgSend;
NSString *__wmConversationId = {conv};
NSString *__wmOutputPath = {out};
SEL __wmRespondsSel = sel_registerName("respondsToSelector:");
SEL __wmIsKindSel = sel_registerName("isKindOfClass:");
SEL __wmClassSel = sel_registerName("class");
NSString* (^__wmString)(id, SEL) = ^NSString*(id __wmObject, SEL __wmSelector) {{
    if (!__wmObject || !__wmResponds(__wmObject, __wmRespondsSel, __wmSelector)) {{ return @""; }}
    id __wmValue = __wmObj(__wmObject, __wmSelector);
    if (!__wmValue) {{ return @""; }}
    if (__wmIsKind(__wmValue, __wmIsKindSel, [NSString class])) {{ return (NSString *)__wmValue; }}
    return @"";
}};
BOOL (^__wmBoolValue)(id, SEL) = ^BOOL(id __wmObject, SEL __wmSelector) {{
    if (!__wmObject || !__wmResponds(__wmObject, __wmRespondsSel, __wmSelector)) {{ return NO; }}
    return __wmBool(__wmObject, __wmSelector);
}};
long long (^__wmIntValue)(id, SEL) = ^long long(id __wmObject, SEL __wmSelector) {{
    if (!__wmObject || !__wmResponds(__wmObject, __wmRespondsSel, __wmSelector)) {{ return 0; }}
    return __wmLl(__wmObject, __wmSelector);
}};
unsigned long long (^__wmUIntValue)(id, SEL) = ^unsigned long long(id __wmObject, SEL __wmSelector) {{
    if (!__wmObject || !__wmResponds(__wmObject, __wmRespondsSel, __wmSelector)) {{ return 0; }}
    return __wmUll(__wmObject, __wmSelector);
}};
NSMutableDictionary *__wmPayload = [NSMutableDictionary dictionary];
NSMutableArray *__wmRows = [NSMutableArray array];
[__wmPayload setObject:__wmConversationId forKey:@"conversation_id"];
[__wmPayload setObject:__wmRows forKey:@"members"];
id __wmMgr = [(id)objc_getClass("WEWServiceManager") defaultServiceManager];
id __wmConversationSvc = __wmObj(__wmMgr, sel_registerName("conversationService"));
SEL __wmConvSel = sel_registerName("conversationWithId:");
if (!__wmConversationSvc || !__wmResponds(__wmConversationSvc, __wmRespondsSel, __wmConvSel)) {{
    [__wmPayload setObject:@"conversation selector unavailable" forKey:@"error"];
}} else {{
    id __wmConversation = __wmObj1(__wmConversationSvc, __wmConvSel, __wmConversationId);
    if (!__wmConversation) {{
        [__wmPayload setObject:@"conversation not found" forKey:@"error"];
    }} else {{
        [__wmPayload setObject:(__wmString(__wmConversation, sel_registerName("name")) ?: @"") forKey:@"conversation_name"];
        [__wmPayload setObject:@(__wmUll(__wmConversation, sel_registerName("lastMessageId"))) forKey:@"conversation_last_message_id"];
        Class __wmDataSourceClass = objc_getClass("WEWConversationMemberDataSource");
        if (!__wmDataSourceClass) {{
            [__wmPayload setObject:@"member data source class unavailable" forKey:@"error"];
        }} else {{
            id __wmDataSource = [[__wmDataSourceClass alloc] init];
            SEL __wmSetConversationSel = sel_registerName("setConversation:");
            SEL __wmFetchMembersSel = sel_registerName("fetchConversationAllUserArr");
            if (!__wmDataSource || !__wmResponds(__wmDataSource, __wmRespondsSel, __wmSetConversationSel) || !__wmResponds(__wmDataSource, __wmRespondsSel, __wmFetchMembersSel)) {{
                [__wmPayload setObject:@"member list selector unavailable" forKey:@"error"];
            }} else {{
                __wmVoid1(__wmDataSource, __wmSetConversationSel, __wmConversation);
                id __wmUsers = __wmObj(__wmDataSource, __wmFetchMembersSel);
                if (!__wmUsers || !__wmIsKind(__wmUsers, __wmIsKindSel, [NSArray class])) {{
                    [__wmPayload setObject:@"member list selector did not return an array" forKey:@"error"];
                }} else {{
                    for (id __wmUser in (NSArray *)__wmUsers) {{
                        if (!__wmUser) {{ continue; }}
                        BOOL __wmHideEmail = __wmBoolValue(__wmUser, sel_registerName("isHideEmail")) || __wmBoolValue(__wmUser, sel_registerName("isExternalHideMail")) || __wmBoolValue(__wmUser, sel_registerName("isHiddenBizMail"));
                        BOOL __wmHideMobile = __wmBoolValue(__wmUser, sel_registerName("isHideMobilePhone")) || __wmBoolValue(__wmUser, sel_registerName("isExternalHideMobile"));
                        BOOL __wmHidePhone = __wmBoolValue(__wmUser, sel_registerName("isHidePhone")) || __wmBoolValue(__wmUser, sel_registerName("isExternalHidePhone"));
                        BOOL __wmHideOfficePhone = __wmBoolValue(__wmUser, sel_registerName("isHideOfficePhone")) || __wmBoolValue(__wmUser, sel_registerName("IsExternalHideOfficePhone"));
                        BOOL __wmHidePosition = __wmBoolValue(__wmUser, sel_registerName("isHidePosition")) || __wmBoolValue(__wmUser, sel_registerName("isExternalHidePosition"));
                        BOOL __wmHideBizMail = __wmBoolValue(__wmUser, sel_registerName("isHiddenBizMail"));
                        NSString *__wmName = __wmString(__wmUser, sel_registerName("name"));
                        NSString *__wmRealName = __wmString(__wmUser, sel_registerName("realName"));
                        NSString *__wmRtxName = __wmString(__wmUser, sel_registerName("rtxName"));
                        NSString *__wmAccount = __wmString(__wmUser, sel_registerName("account"));
                        NSString *__wmDisplayName = [__wmRealName length] ? __wmRealName : ([__wmName length] ? __wmName : ([__wmRtxName length] ? __wmRtxName : __wmAccount));
                        NSMutableDictionary *__wmRow = [NSMutableDictionary dictionary];
                        [__wmRow setObject:@(__wmIntValue(__wmUser, sel_registerName("userId"))) forKey:@"user_id"];
                        [__wmRow setObject:@(__wmUIntValue(__wmUser, sel_registerName("gid"))) forKey:@"gid"];
                        [__wmRow setObject:@(__wmUIntValue(__wmUser, sel_registerName("corpanyId"))) forKey:@"corp_id"];
                        [__wmRow setObject:__wmDisplayName ?: @"" forKey:@"display_name"];
                        [__wmRow setObject:__wmName ?: @"" forKey:@"name"];
                        [__wmRow setObject:__wmRealName ?: @"" forKey:@"real_name"];
                        [__wmRow setObject:__wmRtxName ?: @"" forKey:@"rtx_name"];
                        [__wmRow setObject:__wmAccount ?: @"" forKey:@"account"];
                        [__wmRow setObject:__wmString(__wmUser, sel_registerName("colleageRemark")) ?: @"" forKey:@"colleague_remark"];
                        [__wmRow setObject:__wmString(__wmUser, sel_registerName("namePinyin")) ?: @"" forKey:@"name_pinyin"];
                        [__wmRow setObject:__wmString(__wmUser, sel_registerName("rtxNamePinyin")) ?: @"" forKey:@"rtx_name_pinyin"];
                        [__wmRow setObject:__wmHideEmail ? @"" : (__wmString(__wmUser, sel_registerName("email")) ?: @"") forKey:@"email"];
                        [__wmRow setObject:__wmHideBizMail ? @"" : (__wmString(__wmUser, sel_registerName("bizMail")) ?: @"") forKey:@"biz_mail"];
                        [__wmRow setObject:__wmHideMobile ? @"" : (__wmString(__wmUser, sel_registerName("mobile")) ?: @"") forKey:@"mobile"];
                        [__wmRow setObject:__wmHidePhone ? @"" : (__wmString(__wmUser, sel_registerName("phone")) ?: @"") forKey:@"phone"];
                        [__wmRow setObject:__wmHideOfficePhone ? @"" : (__wmString(__wmUser, sel_registerName("officePhone")) ?: @"") forKey:@"office_phone"];
                        [__wmRow setObject:__wmHidePosition ? @"" : (__wmString(__wmUser, sel_registerName("position")) ?: @"") forKey:@"position"];
                        [__wmRow setObject:__wmString(__wmUser, sel_registerName("externalCompanyName")) ?: @"" forKey:@"external_company_name"];
                        [__wmRow setObject:__wmString(__wmUser, sel_registerName("unionId")) ?: @"" forKey:@"union_id"];
                        [__wmRow setObject:__wmString(__wmUser, sel_registerName("wxOpenId")) ?: @"" forKey:@"wx_open_id"];
                        [__wmRow setObject:@(__wmIntValue(__wmUser, sel_registerName("gender"))) forKey:@"gender"];
                        [__wmRow setObject:@(__wmIntValue(__wmUser, sel_registerName("nameStatus"))) forKey:@"name_status"];
                        [__wmRow setObject:@(__wmUIntValue(__wmUser, sel_registerName("displayOrder"))) forKey:@"display_order"];
                        [__wmRow setObject:@(__wmBoolValue(__wmUser, sel_registerName("isWechatFriend"))) forKey:@"is_wechat_friend"];
                        [__wmRow setObject:@(__wmBoolValue(__wmUser, sel_registerName("isCorpCustomer"))) forKey:@"is_corp_customer"];
                        [__wmRow setObject:@(__wmBoolValue(__wmUser, sel_registerName("isUserUseNickName"))) forKey:@"is_user_use_nick_name"];
                        [__wmRow setObject:@(__wmBoolValue(__wmUser, sel_registerName("isAssociateAdmin"))) forKey:@"is_associate_admin"];
                        [__wmRow setObject:@(__wmBoolValue(__wmUser, sel_registerName("isBizMailAvailable"))) forKey:@"is_biz_mail_available"];
                        [__wmRow setObject:@(__wmBoolValue(__wmUser, sel_registerName("isWXWorkMail"))) forKey:@"is_wx_work_mail"];
                        [__wmRow setObject:@(__wmBoolValue(__wmUser, sel_registerName("isHideDept"))) forKey:@"is_hide_dept"];
                        [__wmRow setObject:@(__wmHideEmail) forKey:@"is_email_hidden"];
                        [__wmRow setObject:@(__wmHideBizMail) forKey:@"is_biz_mail_hidden"];
                        [__wmRow setObject:@(__wmHideMobile) forKey:@"is_mobile_hidden"];
                        [__wmRow setObject:@(__wmHidePhone) forKey:@"is_phone_hidden"];
                        [__wmRow setObject:@(__wmHideOfficePhone) forKey:@"is_office_phone_hidden"];
                        [__wmRow setObject:@(__wmHidePosition) forKey:@"is_position_hidden"];
                        [__wmRow setObject:@(__wmBoolValue(__wmUser, sel_registerName("isExternalHidePersonalCorp"))) forKey:@"is_external_personal_corp_hidden"];
                        [__wmRows addObject:__wmRow];
                    }}
                }}
            }}
        }}
    }}
}}
[__wmPayload setObject:@([__wmRows count]) forKey:@"member_count"];
NSError *__wmJsonError = nil;
NSData *__wmJson = [NSJSONSerialization dataWithJSONObject:__wmPayload options:NSJSONWritingPrettyPrinted error:&__wmJsonError];
if (__wmJson) {{ [__wmJson writeToFile:__wmOutputPath atomically:YES]; }}
"#
    )
}

#[cfg(all(test, target_os = "macos"))]
mod tests {
    use super::*;

    #[test]
    fn redacts_lldb_expression_and_conversation_ids_from_debug_output() {
        let raw = "(lldb) expr -l objc++ -O -- NSString *__wmConversationId = @\"R:123456\";\nerror: failed for R:123456\n";

        let redacted = redact_runtime_debug_output(raw);
        assert!(!redacted.contains("NSString"));
        assert!(!redacted.contains("R:123456"));
        assert!(redacted.contains("<redacted-conversation-id>"));
    }

    #[test]
    fn builds_one_attach_script_for_multiple_expressions() {
        let script = build_lldb_script(["int a = 1;", "int b = 2;"]);

        assert_eq!(script.matches("expr -l objc++ -O --").count(), 2);
        assert_eq!(script.matches("process detach").count(), 1);
        assert!(script.ends_with("quit\n"));
    }
}
