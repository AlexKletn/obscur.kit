#![doc = include_str!("lara_theme.doc/_module.md")]

#[doc = include_str!("lara_theme.doc/LARA_CSS.md")]
pub(super) const LARA_CSS: &str = r"
* { box-sizing: border-box; }
#app { display: flex; flex-direction: column; background-color: #f8f9fa; }

/* ---- Switcher bar ---- */
.kit-switcher { display: flex; flex-direction: row; align-items: center; height: 60px;
    padding-left: 18px; padding-right: 18px; background-color: #ffffff;
    border-bottom: 1px solid #e9ecef; }
.kit-brand { display: flex; flex-direction: row; align-items: center; width: 60px; height: 60px; }
.kit-logo { width: 11px; height: 11px; border-radius: 4px; background-color: #3B82F6; }
.kit-brand-name { width: 40px; height: 16px; margin-left: 8px; color: #343a40;
    font-size: 13px; font-weight: 700; }
.kit-tabs { display: flex; flex-direction: row; }
.kit-tab { display: flex; flex-direction: column; align-items: center; justify-content: center;
    width: 108px; height: 60px; }
.kit-tab-label { height: 16px; color: #9aa3ad; font-size: 12px; font-weight: 600; text-align: center; }
.kit-tab.active .kit-tab-label { color: #343a40; }
.kit-underline { width: 60px; height: 2px; border-radius: 2px; background-color: #3B82F6; margin-top: 6px; }

/* ---- Gallery frame ---- */
.m3-page { display: flex; flex-direction: column; padding: 34px 40px 60px; }
.m3-head { display: flex; flex-direction: column; height: 108px; margin-bottom: 26px; }
.m3-eyebrow { height: 16px; color: #9aa3ad; font-size: 11px; font-weight: 600; }
.m3-title { height: 48px; color: #343a40; font-size: 38px; font-weight: 700; }
.m3-sub { height: 22px; color: #6c757d; font-size: 15px; }

.gallery-grid { display: flex; flex-direction: column; }
.row { display: flex; flex-direction: row; align-items: stretch; margin-bottom: 24px; }
.row.r1 { height: 232px; }
.row.r2 { height: 336px; }
.row.r3 { height: 420px; }
.row.r4 { height: 252px; }

/* Cards: white, 1px grey border, 6px radius, a gentle shadow. */
.card { background-color: #ffffff; border-radius: 6px; padding: 22px;
    border: 1px solid #e9ecef; box-shadow: 0 1px 2px rgba(0,0,0,0.04); }
.card.col { width: 588px; margin-right: 24px; }
.card.full { width: 1200px; margin-bottom: 24px; }
.card.full.table-c { height: 288px; }
.card.full.feedback-c { height: 132px; }
.card-label { height: 16px; margin-bottom: 16px; color: #9aa3ad; font-size: 11px; font-weight: 600; }

/* ---- Buttons: solid blue primary, grey secondary, outlined + text ---- */
.btn-row { display: flex; flex-direction: row; align-items: center; height: 56px; }
.btn { display: flex; flex-direction: row; align-items: center; justify-content: center;
    height: 40px; border-radius: 6px; padding-left: 20px; padding-right: 20px; margin-right: 10px; }
.btn-label { height: 18px; font-size: 14px; font-weight: 600; text-align: center; }
.btn-filled { width: 110px; background-color: #3B82F6; }
.btn-filled .btn-label { color: #ffffff; }
.btn-tonal { width: 104px; background-color: #64748B; }
.btn-tonal .btn-label { color: #ffffff; }
.btn-outlined { width: 128px; background-color: #ffffff; border: 1px solid #3B82F6; }
.btn-outlined .btn-label { color: #3B82F6; }
.btn-text { width: 92px; padding-left: 16px; padding-right: 16px; background-color: #ffffff; }
.btn-text .btn-label { color: #3B82F6; }
.btn-dialog { width: 152px; }
.btn-elevated { width: 132px; background-color: #22C55E; }
.btn-elevated .btn-label { color: #ffffff; }
.btn-disabled { width: 128px; background-color: #a5c9fb; }
.btn-disabled .btn-label { color: #ffffff; }
/* FAB -> the icon-square add affordance in the Lara set. */
.fab { display: flex; flex-direction: row; align-items: center; justify-content: center;
    width: 56px; height: 56px; border-radius: 6px; background-color: #ffffff; border: 1px solid #ced4da; }
.fab-glyph { height: 30px; color: #495057; font-size: 26px; font-weight: 400; text-align: center; }
.btn-sm { height: 38px; }
.btn-wide { width: 168px; }

/* ---- Selection ---- */
.sel-cols { display: flex; flex-direction: row; }
.sel-col { display: flex; flex-direction: column; width: 168px; }
.sel-item { display: flex; flex-direction: row; align-items: center; height: 34px; }
.sel-col .sel-item { margin-bottom: 14px; }
.sel-col.wide-gap .sel-item { margin-bottom: 18px; height: 32px; }
.sel-text { height: 20px; margin-left: 11px; color: #495057; font-size: 14px; }
.checkbox-on { display: flex; align-items: center; justify-content: center;
    width: 20px; height: 20px; border-radius: 6px; background-color: #3B82F6; }
.check-glyph { height: 14px; color: #ffffff; font-size: 12px; text-align: center; }
.checkbox-off { width: 20px; height: 20px; border-radius: 6px; border: 2px solid #ced4da; background-color: #ffffff; }
.checkbox-disabled { width: 20px; height: 20px; border-radius: 6px; border: 2px solid #e9ecef; background-color: #e9ecef; }
.radio-on { display: flex; align-items: center; justify-content: center;
    width: 20px; height: 20px; border-radius: 10px; border: 2px solid #ced4da; background-color: #ffffff; }
.radio-dot { width: 10px; height: 10px; border-radius: 5px; background-color: #3B82F6; }
.radio-off { width: 20px; height: 20px; border-radius: 10px; border: 2px solid #ced4da; background-color: #ffffff; }
.toggle-on { display: flex; flex-direction: row; align-items: center; justify-content: flex-end;
    width: 46px; height: 26px; border-radius: 13px; background-color: #3B82F6; padding: 3px; }
.toggle-knob { width: 20px; height: 20px; border-radius: 10px; background-color: #ffffff; }
.toggle-off { display: flex; flex-direction: row; align-items: center; justify-content: flex-start;
    width: 46px; height: 26px; border-radius: 13px; background-color: #cbd5e1; padding: 3px; }
.toggle-knob-off { width: 20px; height: 20px; border-radius: 10px; background-color: #ffffff; }

/* ---- Text fields: filled white inputs, 6px radius, grey borders ---- */
.field-col { display: flex; flex-direction: column; }
.field { margin-bottom: 16px; }
.field-filled { background-color: #ffffff; border-radius: 6px; border: 1px solid #ced4da; padding: 9px 12px; }
.field-outlined { background-color: #ffffff; border: 1px solid #ced4da; border-radius: 6px; padding: 9px 12px; }
.field-error { background-color: #ffffff; border: 1px solid #EF4444; border-radius: 6px; padding: 9px 12px; }
.field-label-primary { height: 14px; margin-bottom: 4px; color: #495057; font-size: 11px; font-weight: 600; }
.field-label { height: 14px; margin-bottom: 4px; color: #495057; font-size: 11px; font-weight: 600; }
.field-label-error { height: 14px; margin-bottom: 4px; color: #EF4444; font-size: 11px; font-weight: 600; }
.field-value { height: 20px; color: #495057; font-size: 15px; }
.field-placeholder { height: 20px; color: #9aa3ad; font-size: 15px; }
.field-error-text { height: 14px; margin-top: 5px; color: #EF4444; font-size: 11px; }

/* ---- Menu / dropdown ---- */
.select-box { display: flex; flex-direction: row; align-items: center; justify-content: space-between;
    width: 260px; height: 44px; background-color: #ffffff; border: 1px solid #ced4da; border-radius: 6px; padding: 0 14px; }
.select-value { height: 20px; color: #495057; font-size: 15px; }
.select-chevron { width: 16px; height: 20px; color: #9aa3ad; font-size: 14px; text-align: right; }
.menu-panel { display: flex; flex-direction: column; width: 260px; height: 136px; margin-top: 8px;
    background-color: #ffffff; border: 1px solid #e9ecef; border-radius: 6px; padding-top: 6px; padding-bottom: 6px;
    box-shadow: 0 10px 28px rgba(0,0,0,0.12); }
.menu-item { display: flex; flex-direction: row; align-items: center; justify-content: space-between;
    height: 40px; padding: 0 12px; }
.menu-item.selected { background-color: #eff6ff; border-radius: 6px; }
.menu-text { height: 20px; color: #495057; font-size: 15px; }
.menu-check { width: 16px; height: 20px; color: #3B82F6; font-size: 14px; text-align: right; }
.chips-row { display: flex; flex-direction: row; align-items: center; height: 32px; margin-top: 16px; }
.chip { display: flex; flex-direction: row; align-items: center; justify-content: center; height: 32px;
    border-radius: 16px; padding-left: 14px; padding-right: 14px; margin-right: 8px; }
.chip-filled { width: 112px; background-color: #dee2e6; }
.chip-filled .chip-text { color: #495057; }
.chip-check { height: 16px; margin-right: 6px; color: #6c757d; font-size: 12px; }
.chip-outlined { width: 104px; background-color: #ffffff; border: 1px solid #dee2e6; }
.chip-outlined .chip-text { color: #495057; }
.chip-tonal { background-color: #dee2e6; }
.chip-tonal .chip-text { color: #495057; }
.chip-wide { width: 172px; }
.chip-text { height: 18px; font-size: 14px; font-weight: 600; }

/* ---- Card & list ---- */
.mini-card { display: flex; flex-direction: column; height: 208px; background-color: #ffffff;
    border: 1px solid #e9ecef; border-radius: 6px; box-shadow: 0 1px 3px rgba(0,0,0,0.06); }
/* cover hatch -> flat grey fill (repeating-linear-gradient unsupported). */
.mini-cover { display: flex; align-items: center; justify-content: center; height: 84px;
    border-radius: 6px 6px 0 0; background-color: #eef1f4; }
.mini-cover-text { height: 16px; color: #adb5bd; font-size: 11px; text-align: center; }
.mini-body { display: flex; flex-direction: column; height: 124px; padding: 16px; }
.mini-title { height: 22px; color: #343a40; font-size: 16px; font-weight: 700; }
.mini-sub { height: 18px; margin-top: 4px; color: #6c757d; font-size: 13px; }
.mini-actions { display: flex; flex-direction: row; align-items: center; height: 38px; margin-top: 14px; }
.people { display: flex; flex-direction: column; height: 100px; margin-top: 14px; }
.person { display: flex; flex-direction: row; align-items: center; height: 50px; }
.person-meta { display: flex; flex-direction: column; justify-content: center; height: 50px;
    margin-left: 13px; width: 420px; }
.person-name { height: 18px; color: #495057; font-size: 14px; }
.person-email { height: 16px; margin-top: 2px; color: #9aa3ad; font-size: 12px; }
.person-menu { width: 24px; height: 20px; color: #9aa3ad; font-size: 18px; text-align: right; }
.avatar { display: flex; align-items: center; justify-content: center;
    width: 36px; height: 36px; border-radius: 18px; }
.avatar-text { height: 16px; font-size: 13px; font-weight: 600; text-align: center; color: #ffffff; }
.avatar-ak { background-color: #3B82F6; }
.avatar-mr { background-color: #22C55E; }
.avatar-jp { background-color: #8b5cf6; }
.avatar-more { background-color: #e9ecef; }
.avatar-more .avatar-text { color: #6c757d; }

/* ---- Navigation ---- */
.breadcrumb { display: flex; flex-direction: row; align-items: center; height: 20px; margin-bottom: 18px; }
.crumb { height: 18px; margin-right: 8px; color: #3B82F6; font-size: 13px; }
.crumb-sep { height: 18px; margin-right: 8px; color: #ced4da; font-size: 13px; }
.crumb-active { height: 18px; color: #495057; font-size: 13px; font-weight: 600; }
.nav-tabs { display: flex; flex-direction: row; height: 42px;
    border-bottom: 2px solid #e9ecef; margin-bottom: 14px; }
.nav-tab { display: flex; flex-direction: column; align-items: center; justify-content: flex-end;
    width: 96px; }
.nav-tab-label { height: 24px; color: #6c757d; font-size: 14px; font-weight: 600; text-align: center; }
.nav-tab-label.active { color: #3B82F6; }
.nav-tab-underline { width: 96px; height: 2px; background-color: #3B82F6; }
.nav-list { display: flex; flex-direction: column; width: 260px; }
.nav-item { display: flex; flex-direction: row; align-items: center; height: 40px;
    border-radius: 6px; padding-left: 12px; padding-right: 12px; margin-bottom: 4px; }
.nav-item.active { background-color: #eff6ff; }
.nav-item-text { height: 20px; margin-left: 10px; color: #495057; font-size: 14px; }
.nav-item.active .nav-item-text { color: #1d4ed8; font-weight: 600; }
.dot-on { width: 6px; height: 6px; border-radius: 3px; background-color: #3B82F6; }
.dot-off { width: 6px; height: 6px; border-radius: 3px; background-color: #ced4da; }

/* ---- Badges / chips / avatars: bold solid tags with white text, 6px radius ---- */
.badge-row { display: flex; flex-direction: row; align-items: center; height: 26px; margin-bottom: 22px; }
.badge { display: flex; align-items: center; justify-content: center; height: 24px;
    border-radius: 6px; margin-right: 8px; }
.badge-text { height: 16px; font-size: 12px; font-weight: 700; text-align: center; }
.badge-success { width: 76px; background-color: #22C55E; }
.badge-success .badge-text { color: #ffffff; }
.badge-warn { width: 84px; background-color: #F59E0B; }
.badge-warn .badge-text { color: #ffffff; }
.badge-error { width: 66px; background-color: #EF4444; }
.badge-error .badge-text { color: #ffffff; }
.badge-neutral { width: 64px; background-color: #e9ecef; }
.badge-neutral .badge-text { color: #6c757d; }
.badge-count { display: flex; align-items: center; justify-content: center;
    width: 24px; height: 20px; border-radius: 10px; background-color: #EF4444; }
.badge-count-text { height: 14px; color: #ffffff; font-size: 11px; font-weight: 700; text-align: center; }
.avatar-stack { display: flex; flex-direction: row; align-items: center; height: 40px; }
.avatar.stack { border: 2px solid #ffffff; }
.avatar.overlap { margin-left: -10px; }

/* ---- Sliders / progress ---- */
.slider-head { display: flex; flex-direction: row; align-items: center; justify-content: space-between;
    height: 18px; margin-bottom: 10px; }
.slider-name { height: 16px; color: #6c757d; font-size: 13px; }
.slider-val { height: 16px; color: #3B82F6; font-size: 13px; font-weight: 600; text-align: right; }
.slider { position: relative; height: 24px; margin-bottom: 24px; }
.slider-track { position: absolute; left: 0; right: 0; top: 9px; height: 6px;
    border-radius: 3px; background-color: #dee2e6; }
.slider-fill { position: absolute; left: 0; top: 9px; width: 55%; height: 6px;
    border-radius: 3px; background-color: #3B82F6; }
.slider-knob { position: absolute; left: 53%; top: 3px; width: 18px; height: 18px;
    border-radius: 9px; background-color: #3B82F6; box-shadow: 0 0 0 4px rgba(59,130,246,0.20); }
.progress-label { height: 18px; margin-bottom: 8px; color: #6c757d; font-size: 13px; }
.progress { height: 8px; border-radius: 4px; background-color: #dee2e6; margin-bottom: 20px; }
.progress-fill { width: 72%; height: 8px; border-radius: 4px; background-color: #3B82F6; }
.spinner-row { display: flex; flex-direction: row; align-items: center; height: 28px; }
.spinner { width: 24px; height: 24px; border-radius: 12px; margin-right: 14px;
    border: 3px solid #dee2e6; border-top-color: #3B82F6; }
.spinner-text { height: 18px; color: #6c757d; font-size: 13px; }

/* ---- Data table ---- */
.table { border: 1px solid #e9ecef; border-radius: 6px; }
.thead { display: grid; grid-template-columns: 1.6fr 1fr 1fr 0.8fr;
    background-color: #f8f9fa; padding: 13px 16px; border-radius: 6px 6px 0 0; }
.th { height: 16px; color: #343a40; font-size: 12px; font-weight: 700; }
.th-right { text-align: right; }
.trow { display: grid; grid-template-columns: 1.6fr 1fr 1fr 0.8fr; align-items: center;
    padding: 13px 16px; border-top: 1px solid #f1f3f5; }
.td { height: 20px; color: #495057; font-size: 14px; }
.td-name { color: #343a40; font-weight: 600; }
.td-muted { color: #6c757d; }
.td-mono { color: #495057; }
.td-right { text-align: right; }

/* ---- Feedback ---- */
.feedback-row { display: flex; flex-direction: row; align-items: center; height: 40px; }
";
