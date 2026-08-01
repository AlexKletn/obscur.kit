#![doc = include_str!("dark_saas_theme.doc/_module.md")]

#[doc = include_str!("dark_saas_theme.doc/DARK_SAAS_CSS.md")]
pub(super) const DARK_SAAS_CSS: &str = r"
* { box-sizing: border-box; }
#app { display: flex; flex-direction: column; background-color: #0c0e14; }

/* ---- Switcher bar ---- */
.kit-switcher { display: flex; flex-direction: row; align-items: center; height: 60px;
    padding-left: 18px; padding-right: 18px; background-color: #14161f;
    border-bottom: 1px solid #232634; }
.kit-brand { display: flex; flex-direction: row; align-items: center; width: 60px; height: 60px; }
.kit-logo { width: 11px; height: 11px; border-radius: 4px; background-color: #7c5cff; }
.kit-brand-name { width: 40px; height: 16px; margin-left: 8px; color: #e6e8ee;
    font-size: 13px; font-weight: 700; }
.kit-tabs { display: flex; flex-direction: row; }
.kit-tab { display: flex; flex-direction: column; align-items: center; justify-content: center;
    width: 108px; height: 60px; }
.kit-tab-label { height: 16px; color: #6b7180; font-size: 12px; font-weight: 500; text-align: center; }
.kit-tab.active .kit-tab-label { color: #e6e8ee; }
.kit-underline { width: 60px; height: 2px; border-radius: 2px; background-color: #7c5cff; margin-top: 6px; }

/* ---- Gallery frame ---- */
.m3-page { display: flex; flex-direction: column; padding: 34px 40px 60px; }
.m3-head { display: flex; flex-direction: column; height: 108px; margin-bottom: 26px; }
.m3-eyebrow { height: 16px; color: #6b7180; font-size: 11px; font-weight: 500; }
/* Title text-gradient (#fff->#b9b4ff) is GPU-class -> solid light fill. */
.m3-title { height: 48px; color: #e9e7ff; font-size: 38px; font-weight: 700; }
.m3-sub { height: 22px; color: #8b90a0; font-size: 15px; }

.gallery-grid { display: flex; flex-direction: column; }
.row { display: flex; flex-direction: row; align-items: stretch; margin-bottom: 24px; }
.row.r1 { height: 232px; }
.row.r2 { height: 336px; }
.row.r3 { height: 420px; }
.row.r4 { height: 252px; }

/* Cards: raised near-black surfaces with a subtle light-alpha hairline. */
.card { background-color: #14161f; border-radius: 14px; padding: 22px;
    border: 1px solid #232634; }
.card.col { width: 588px; margin-right: 24px; }
.card.full { width: 1200px; margin-bottom: 24px; }
.card.full.table-c { height: 288px; }
.card.full.feedback-c { height: 132px; }
.card-label { height: 16px; margin-bottom: 16px; color: #6b7180; font-size: 11px; font-weight: 500; }

/* ---- Buttons: violet primary, slate secondary, ghost accent-text ---- */
.btn-row { display: flex; flex-direction: row; align-items: center; height: 56px; }
.btn { display: flex; flex-direction: row; align-items: center; justify-content: center;
    height: 40px; border-radius: 9px; padding-left: 20px; padding-right: 20px; margin-right: 10px; }
.btn-label { height: 18px; font-size: 14px; font-weight: 600; text-align: center; }
.btn-filled { width: 110px; background-color: #7c5cff; }
.btn-filled .btn-label { color: #ffffff; }
.btn-tonal { width: 104px; background-color: #1c1f2b; border: 1px solid #2e3242; }
.btn-tonal .btn-label { color: #e6e8ee; }
.btn-outlined { width: 128px; background-color: #14161f; border: 1px solid #2e3242; }
.btn-outlined .btn-label { color: #b9b4ff; }
.btn-text { width: 92px; padding-left: 16px; padding-right: 16px; background-color: #14161f; }
.btn-text .btn-label { color: #b9b4ff; }
.btn-dialog { width: 152px; }
.btn-elevated { width: 132px; background-color: #1c1f2b; border: 1px solid #2e3242; }
.btn-elevated .btn-label { color: #e6e8ee; }
.btn-disabled { width: 128px; background-color: #171922; border: 1px solid #20232e; }
.btn-disabled .btn-label { color: #4a4f60; }
/* FAB -> the primary accent add-button (glow degrades on CPU). */
.fab { display: flex; flex-direction: row; align-items: center; justify-content: center;
    width: 56px; height: 56px; border-radius: 10px; background-color: #7c5cff;
    box-shadow: 0 6px 20px rgba(124,92,255,0.45); }
.fab-glyph { height: 30px; color: #ffffff; font-size: 26px; font-weight: 400; text-align: center; }
.btn-sm { height: 38px; }
.btn-wide { width: 168px; }

/* ---- Selection ---- */
.sel-cols { display: flex; flex-direction: row; }
.sel-col { display: flex; flex-direction: column; width: 168px; }
.sel-item { display: flex; flex-direction: row; align-items: center; height: 34px; }
.sel-col .sel-item { margin-bottom: 14px; }
.sel-col.wide-gap .sel-item { margin-bottom: 18px; height: 32px; }
.sel-text { height: 20px; margin-left: 11px; color: #e6e8ee; font-size: 14px; }
.checkbox-on { display: flex; align-items: center; justify-content: center;
    width: 18px; height: 18px; border-radius: 6px; background-color: #7c5cff; }
.check-glyph { height: 14px; color: #ffffff; font-size: 12px; text-align: center; }
.checkbox-off { width: 18px; height: 18px; border-radius: 6px; border: 1px solid #3a3f52;
    background-color: #1c1f2b; }
.checkbox-disabled { width: 18px; height: 18px; border-radius: 6px; border: 1px solid #20232e;
    background-color: #171922; }
.radio-on { display: flex; align-items: center; justify-content: center;
    width: 20px; height: 20px; border-radius: 10px; border: 1px solid #3a3f52; background-color: #1c1f2b; }
.radio-dot { width: 10px; height: 10px; border-radius: 5px; background-color: #7c5cff; }
.radio-off { width: 20px; height: 20px; border-radius: 10px; border: 1px solid #3a3f52; background-color: #1c1f2b; }
.toggle-on { display: flex; flex-direction: row; align-items: center; justify-content: flex-end;
    width: 44px; height: 24px; border-radius: 12px; background-color: #7c5cff; padding: 3px; }
.toggle-knob { width: 18px; height: 18px; border-radius: 9px; background-color: #ffffff; }
.toggle-off { display: flex; flex-direction: row; align-items: center; justify-content: flex-start;
    width: 44px; height: 24px; border-radius: 12px; background-color: #2e3242; padding: 3px; }
.toggle-knob-off { width: 18px; height: 18px; border-radius: 9px; background-color: #8b90a0; }

/* ---- Text fields: recessed dark wells ---- */
.field-col { display: flex; flex-direction: column; }
.field { margin-bottom: 16px; }
.field-filled { background-color: #0e1017; border-radius: 9px; border: 1px solid #2e3242; padding: 9px 12px; }
.field-outlined { background-color: #0e1017; border: 1px solid #2e3242; border-radius: 9px; padding: 9px 12px; }
.field-error { background-color: #1c1216; border: 1px solid #d24f4f; border-radius: 9px; padding: 9px 12px; }
.field-label-primary { height: 14px; margin-bottom: 4px; color: #8b90a0; font-size: 11px; }
.field-label { height: 14px; margin-bottom: 4px; color: #8b90a0; font-size: 11px; }
.field-label-error { height: 14px; margin-bottom: 4px; color: #ff8b8b; font-size: 11px; }
.field-value { height: 20px; color: #e6e8ee; font-size: 15px; }
.field-placeholder { height: 20px; color: #6b7180; font-size: 15px; }
.field-error-text { height: 14px; margin-top: 5px; color: #ff8b8b; font-size: 11px; }

/* ---- Menu / dropdown ---- */
.select-box { display: flex; flex-direction: row; align-items: center; justify-content: space-between;
    width: 260px; height: 44px; background-color: #0e1017; border: 1px solid #2e3242; border-radius: 9px; padding: 0 14px; }
.select-value { height: 20px; color: #e6e8ee; font-size: 15px; }
.select-chevron { width: 16px; height: 20px; color: #6b7180; font-size: 14px; text-align: right; }
.menu-panel { display: flex; flex-direction: column; width: 260px; height: 136px; margin-top: 8px;
    background-color: #1a1d28; border: 1px solid #2e3242; border-radius: 9px; padding-top: 6px; padding-bottom: 6px;
    box-shadow: 0 12px 30px rgba(0,0,0,0.5); }
.menu-item { display: flex; flex-direction: row; align-items: center; justify-content: space-between;
    height: 40px; padding: 0 12px; }
.menu-item.selected { background-color: #242838; border-radius: 6px; }
.menu-text { height: 20px; color: #e6e8ee; font-size: 15px; }
.menu-check { width: 16px; height: 20px; color: #b9b4ff; font-size: 14px; text-align: right; }
.chips-row { display: flex; flex-direction: row; align-items: center; height: 32px; margin-top: 16px; }
.chip { display: flex; flex-direction: row; align-items: center; justify-content: center; height: 32px;
    border-radius: 8px; padding-left: 12px; padding-right: 12px; margin-right: 8px; }
.chip-filled { width: 112px; background-color: #1c1f2b; border: 1px solid #2e3242; }
.chip-filled .chip-text { color: #e6e8ee; }
.chip-check { height: 16px; margin-right: 6px; color: #6b7180; font-size: 12px; }
.chip-outlined { width: 104px; background-color: #14161f; border: 1px solid #2e3242; }
.chip-outlined .chip-text { color: #e6e8ee; }
.chip-tonal { background-color: rgba(124,92,255,0.14); border: 1px solid #2e3242; }
.chip-tonal .chip-text { color: #cfc9ff; }
.chip-wide { width: 172px; }
.chip-text { height: 18px; font-size: 14px; font-weight: 500; }

/* ---- Card & list ---- */
.mini-card { display: flex; flex-direction: column; height: 208px; background-color: #0e1017;
    border: 1px solid #232634; border-radius: 12px; }
/* cover hatch -> flat dark fill (repeating-linear-gradient unsupported). */
.mini-cover { display: flex; align-items: center; justify-content: center; height: 84px;
    border-radius: 12px 12px 0 0; background-color: #1a1d28; }
.mini-cover-text { height: 16px; color: #4a4f60; font-size: 11px; text-align: center; }
.mini-body { display: flex; flex-direction: column; height: 124px; padding: 16px; }
.mini-title { height: 22px; color: #e6e8ee; font-size: 16px; font-weight: 600; }
.mini-sub { height: 18px; margin-top: 4px; color: #8b90a0; font-size: 13px; }
.mini-actions { display: flex; flex-direction: row; align-items: center; height: 38px; margin-top: 14px; }
.people { display: flex; flex-direction: column; height: 100px; margin-top: 14px; }
.person { display: flex; flex-direction: row; align-items: center; height: 50px; }
.person-meta { display: flex; flex-direction: column; justify-content: center; height: 50px;
    margin-left: 13px; width: 420px; }
.person-name { height: 18px; color: #e6e8ee; font-size: 14px; }
.person-email { height: 16px; margin-top: 2px; color: #8b90a0; font-size: 12px; }
.person-menu { width: 24px; height: 20px; color: #6b7180; font-size: 18px; text-align: right; }
.avatar { display: flex; align-items: center; justify-content: center;
    width: 36px; height: 36px; border-radius: 18px; }
.avatar-text { height: 16px; font-size: 13px; font-weight: 600; text-align: center; color: #ffffff; }
.avatar-ak { background-color: #c26a52; }
.avatar-mr { background-color: #2fa06e; }
.avatar-jp { background-color: #7b6fce; }
.avatar-more { background-color: #1c1f2b; }
.avatar-more .avatar-text { color: #8b90a0; }

/* ---- Navigation ---- */
.breadcrumb { display: flex; flex-direction: row; align-items: center; height: 20px; margin-bottom: 18px; }
.crumb { height: 18px; margin-right: 8px; color: #8b90a0; font-size: 13px; }
.crumb-sep { height: 18px; margin-right: 8px; color: #3a3f52; font-size: 13px; }
.crumb-active { height: 18px; color: #e6e8ee; font-size: 13px; font-weight: 500; }
.nav-tabs { display: flex; flex-direction: row; height: 42px;
    border-bottom: 1px solid #232634; margin-bottom: 14px; }
.nav-tab { display: flex; flex-direction: column; align-items: center; justify-content: flex-end;
    width: 96px; }
.nav-tab-label { height: 24px; color: #6b7180; font-size: 14px; font-weight: 600; text-align: center; }
.nav-tab-label.active { color: #e6e8ee; }
.nav-tab-underline { width: 80px; height: 2px; border-radius: 2px; background-color: #7c5cff; }
.nav-list { display: flex; flex-direction: column; width: 260px; }
.nav-item { display: flex; flex-direction: row; align-items: center; height: 40px;
    border-radius: 8px; padding-left: 12px; padding-right: 12px; margin-bottom: 4px; }
.nav-item.active { background-color: rgba(124,92,255,0.14); }
.nav-item-text { height: 20px; margin-left: 10px; color: #8b90a0; font-size: 14px; }
.nav-item.active .nav-item-text { color: #cfc9ff; font-weight: 600; }
.dot-on { width: 6px; height: 6px; border-radius: 3px; background-color: #7c5cff; }
.dot-off { width: 6px; height: 6px; border-radius: 3px; background-color: #3a3f52; }

/* ---- Badges / chips / avatars: translucent status pills ---- */
.badge-row { display: flex; flex-direction: row; align-items: center; height: 26px; margin-bottom: 22px; }
.badge { display: flex; align-items: center; justify-content: center; height: 24px;
    border-radius: 20px; margin-right: 8px; }
.badge-text { height: 16px; font-size: 12px; font-weight: 600; text-align: center; }
.badge-success { width: 76px; background-color: rgba(52,211,153,0.14); border: 1px solid rgba(52,211,153,0.25); }
.badge-success .badge-text { color: #6ee7b7; }
.badge-warn { width: 84px; background-color: rgba(251,191,36,0.14); border: 1px solid rgba(251,191,36,0.25); }
.badge-warn .badge-text { color: #fcd34d; }
.badge-error { width: 66px; background-color: rgba(248,113,113,0.14); border: 1px solid rgba(248,113,113,0.25); }
.badge-error .badge-text { color: #fca5a5; }
.badge-neutral { width: 64px; background-color: #1c1f2b; border: 1px solid #2e3242; }
.badge-neutral .badge-text { color: #8b90a0; }
.badge-count { display: flex; align-items: center; justify-content: center;
    width: 24px; height: 20px; border-radius: 6px; background-color: #7c5cff; }
.badge-count-text { height: 14px; color: #ffffff; font-size: 11px; font-weight: 700; text-align: center; }
.avatar-stack { display: flex; flex-direction: row; align-items: center; height: 40px; }
.avatar.stack { border: 2px solid #14161f; }
.avatar.overlap { margin-left: -9px; }

/* ---- Sliders / progress ---- */
.slider-head { display: flex; flex-direction: row; align-items: center; justify-content: space-between;
    height: 18px; margin-bottom: 10px; }
.slider-name { height: 16px; color: #8b90a0; font-size: 13px; }
.slider-val { height: 16px; color: #cfc9ff; font-size: 13px; text-align: right; }
.slider { position: relative; height: 24px; margin-bottom: 24px; }
.slider-track { position: absolute; left: 0; right: 0; top: 9px; height: 5px;
    border-radius: 3px; background-color: #2e3242; }
.slider-fill { position: absolute; left: 0; top: 9px; width: 55%; height: 5px;
    border-radius: 3px; background-color: #7c5cff; }
.slider-knob { position: absolute; left: 53%; top: 3px; width: 17px; height: 17px;
    border-radius: 9px; background-color: #ffffff; box-shadow: 0 0 0 4px rgba(124,92,255,0.25); }
.progress-label { height: 18px; margin-bottom: 8px; color: #8b90a0; font-size: 13px; }
.progress { height: 8px; border-radius: 4px; background-color: #2e3242; margin-bottom: 20px; }
.progress-fill { width: 72%; height: 8px; border-radius: 4px; background-color: #7c5cff; }
.spinner-row { display: flex; flex-direction: row; align-items: center; height: 28px; }
.spinner { width: 24px; height: 24px; border-radius: 12px; margin-right: 14px;
    border: 3px solid #2e3242; border-top-color: #7c5cff; }
.spinner-text { height: 18px; color: #8b90a0; font-size: 13px; }

/* ---- Data table ---- */
.table { border: 1px solid #232634; border-radius: 10px; }
.thead { display: grid; grid-template-columns: 1.6fr 1fr 1fr 0.8fr;
    background-color: #0e1017; padding: 12px 16px; border-radius: 10px 10px 0 0; }
.th { height: 16px; color: #6b7180; font-size: 12px; font-weight: 600; }
.th-right { text-align: right; }
.trow { display: grid; grid-template-columns: 1.6fr 1fr 1fr 0.8fr; align-items: center;
    padding: 12px 16px; border-top: 1px solid #1c1f2b; }
.td { height: 20px; color: #e6e8ee; font-size: 14px; }
.td-name { font-weight: 500; }
.td-muted { color: #8b90a0; }
.td-mono { color: #cfc9ff; }
.td-right { text-align: right; }

/* ---- Feedback ---- */
.feedback-row { display: flex; flex-direction: row; align-items: center; height: 40px; }
";
