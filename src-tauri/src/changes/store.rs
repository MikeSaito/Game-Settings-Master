use super::*;
#[derive(Clone)]
pub(super) struct Plan {
    pub(super) gpu: crate::gpu::GpuCapabilities,
    pub(super) public: PreparedChanges,
    pub(super) request: PrepareRequest,
    pub(super) profile: GameProfile,
    pub(super) profile_revision: String,
    pub(super) active_dir: PathBuf,
    pub(super) originals: BTreeMap<String, Option<Vec<u8>>>,
    pub(super) replacements: BTreeMap<String, Option<Vec<u8>>>,
    pub(super) created: Instant,
}

pub(super) fn plans() -> &'static Mutex<HashMap<String, Plan>> {
    static PLANS: OnceLock<Mutex<HashMap<String, Plan>>> = OnceLock::new();
    PLANS.get_or_init(|| Mutex::new(HashMap::new()))
}
