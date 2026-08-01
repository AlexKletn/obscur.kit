#![doc = include_str!("neutral_theme.doc/_module.md")]

#[doc = include_str!("neutral_theme.doc/NEUTRAL_CSS.md")]
pub(super) const NEUTRAL_CSS: &str = r"
* { box-sizing: border-box; }
#app { display: flex; flex-direction: column; background-color: #f4f4f5; }
.kit-switcher { display: flex; flex-direction: row; align-items: center; height: 60px;
    padding-left: 18px; padding-right: 18px; background-color: #e4e4e7;
    border-bottom: 1px solid #d4d4d8; }
.kit-brand { display: flex; flex-direction: row; align-items: center; width: 60px; height: 60px; }
.kit-logo { width: 11px; height: 11px; border-radius: 3px; background-color: #52525b; }
.kit-brand-name { width: 40px; height: 16px; margin-left: 8px; color: #27272a; font-size: 13px; font-weight: 700; }
.kit-tabs { display: flex; flex-direction: row; }
.kit-tab { display: flex; flex-direction: column; align-items: center; justify-content: center;
    width: 108px; height: 60px; }
.kit-tab-label { height: 16px; color: #71717a; font-size: 12px; font-weight: 500; text-align: center; }
.kit-tab.active .kit-tab-label { color: #18181b; }
.kit-underline { width: 60px; height: 2px; border-radius: 2px; background-color: #18181b; margin-top: 6px; }
.m3-page { display: flex; flex-direction: column; padding: 34px 40px 60px; }
.m3-head { display: flex; flex-direction: column; height: 108px; margin-bottom: 26px; }
.m3-eyebrow { height: 16px; color: #71717a; font-size: 11px; font-weight: 500; }
.m3-title { height: 48px; color: #18181b; font-size: 38px; font-weight: 700; }
.m3-sub { height: 22px; color: #52525b; font-size: 15px; }
.gallery-grid { display: flex; flex-direction: column; }
.row { display: flex; flex-direction: row; align-items: stretch; margin-bottom: 24px; }
.row.r1 { height: 210px; }
.row.r2 { height: 246px; }
.row.r3 { height: 320px; }
.row.r4 { height: 214px; }
.card { background-color: #ffffff; border-radius: 8px; padding: 22px; border: 1px solid #e4e4e7; }
.card.col { width: 588px; margin-right: 24px; }
.card.full { width: 1200px; margin-bottom: 24px; }
.card-label { height: 16px; margin-bottom: 16px; color: #71717a; font-size: 11px; font-weight: 600; }
.btn-row { display: flex; flex-direction: row; align-items: center; height: 56px; }
.btn { display: flex; align-items: center; justify-content: center; height: 36px; border-radius: 6px;
    padding-left: 16px; padding-right: 16px; margin-right: 10px; background-color: #e4e4e7; }
.btn-label { height: 18px; font-size: 14px; font-weight: 500; text-align: center; color: #27272a; }
";
