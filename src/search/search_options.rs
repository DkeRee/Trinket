pub enum Options {
	AspirationWindow(i32),
	MaxDepthRfp(i32),
	MultiplierRfp(i32),
	NmpReductionBase(i32),
	NmpXShift(i32),
	NmpYStretch(i32),
	LmrDepthLimit(i32),
	LmrFullSearchedMoveLimit(i32),
	IidDepthMin(i32),
	LmpDepthMax(i32),
	LmpMultiplier(i32),
	HistoryDepthMin(i32),
	HistoryPruneMoveLimit(i32),
	HistoryThreshold(i32),
	HistoryReduction(i32)
}

pub struct SearchOptions {
	pub aspiration_window: i32,
	pub max_depth_rfp: i32,
	pub multiplier_rfp: i32,
	pub nmp_reduction_base: i32,
	pub nmp_xshift: i32,
	pub nmp_ystretch: i32,
	pub lmr_depth_limit: i32,
	pub lmr_full_searched_move_limit: i32,
	pub iid_depth_min: i32,
	pub lmp_depth_max: i32,
	pub lmp_multiplier: i32,
	pub history_depth_min: i32,
	pub history_prune_move_limit: i32,
	pub history_threshold: i32,
	pub history_reduction: i32
}

impl SearchOptions {
	pub fn new() -> SearchOptions {
		SearchOptions {
			aspiration_window: Self::ASPIRATION_WINDOW,
			max_depth_rfp: Self::MAX_DEPTH_RFP,
			multiplier_rfp: Self::MULTIPLIER_RFP,
			nmp_reduction_base: Self::NMP_REDUCTION_BASE,
			nmp_xshift: Self::NMP_XSHIFT,
			nmp_ystretch: Self::NMP_YSTRETCH,
			lmr_depth_limit: Self::LMR_DEPTH_LIMIT,
			lmr_full_searched_move_limit: Self::LMR_FULL_SEARCHED_MOVE_LIMIT,
			iid_depth_min: Self::IID_DEPTH_MIN,
			lmp_depth_max: Self::LMP_DEPTH_MAX,
			lmp_multiplier: Self::LMP_MULTIPLIER,
			history_depth_min: Self::HISTORY_DEPTH_MIN,
			history_prune_move_limit: Self::HISTORY_PRUNE_MOVE_LIMIT,
			history_threshold: Self::HISTORY_THRESHOLD,
			history_reduction: Self::HISTORY_REDUCTION
		}
	}

	pub fn get(name: &str, v: i32) -> Options {
		match name {
			"AspirationWindow" => Options::AspirationWindow(v),
			"MaxDepthRfp" => Options::MaxDepthRfp(v),
			"MultiplierRfp" => Options::MultiplierRfp(v),
			"NmpReductionBase" => Options::NmpReductionBase(v),
			"NmpXShift" => Options::NmpXShift(v),
			"NmpYStretch" => Options::NmpYStretch(v),
			"LmrDepthLimit" => Options::LmrDepthLimit(v),
			"LmrFullSearchedMoveLimit" => Options::LmrFullSearchedMoveLimit(v),
			"IidDepthMin" => Options::IidDepthMin(v),
			"LmpDepthMax" => Options::LmpDepthMax(v),
			"LmpMultiplier" => Options::LmpMultiplier(v),
			"HistoryDepthMin" => Options::HistoryDepthMin(v),
			"HistoryPruneMoveLimit" => Options::HistoryPruneMoveLimit(v),
			"HistoryThreshold" => Options::HistoryThreshold(v),
			"HistoryReduction" => Options::HistoryReduction(v),
			_ => panic!()
		}
	}

	pub fn change(&mut self, option: Options) {
		match option {
			Options::AspirationWindow(v) => self.aspiration_window = v,
			Options::MaxDepthRfp(v) => self.max_depth_rfp = v,
			Options::MultiplierRfp(v) => self.multiplier_rfp = v,
			Options::NmpReductionBase(v) => self.nmp_reduction_base = v,
			Options::NmpXShift(v) => self.nmp_xshift = v,
			Options::NmpYStretch(v) => self.nmp_ystretch = v,
			Options::LmrDepthLimit(v) => self.lmr_depth_limit = v,
			Options::LmrFullSearchedMoveLimit(v) => self.lmr_full_searched_move_limit = v,
			Options::IidDepthMin(v) => self.iid_depth_min = v,
			Options::LmpDepthMax(v) => self.lmp_depth_max = v,
			Options::LmpMultiplier(v) => self.lmp_multiplier = v,
			Options::HistoryDepthMin(v) => self.history_depth_min = v,
			Options::HistoryPruneMoveLimit(v) => self.history_prune_move_limit = v,
			Options::HistoryThreshold(v) => self.history_threshold = v,
			Options::HistoryReduction(v) => self.history_reduction = v
		};
	}
}

impl SearchOptions {
	pub const ASPIRATION_WINDOW: i32 = 38;
	pub const MAX_DEPTH_RFP: i32 = 5;
	pub const MULTIPLIER_RFP: i32 = 141;
	pub const NMP_REDUCTION_BASE: i32 = 2;
	pub const NMP_XSHIFT: i32 = 8;
	pub const NMP_YSTRETCH: i32 = 7;
	pub const LMR_DEPTH_LIMIT: i32 = 2;
	pub const LMR_FULL_SEARCHED_MOVE_LIMIT: i32 = 2;
	pub const IID_DEPTH_MIN: i32 = 2;
	pub const LMP_DEPTH_MAX: i32 = 10;
	pub const LMP_MULTIPLIER: i32 = 9;
	pub const HISTORY_DEPTH_MIN: i32 = 4;
	pub const HISTORY_PRUNE_MOVE_LIMIT: i32 = 3;
	pub const HISTORY_THRESHOLD: i32 = 37;
	pub const HISTORY_REDUCTION: i32 = 1;
}