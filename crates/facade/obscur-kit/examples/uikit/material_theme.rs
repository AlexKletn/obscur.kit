#![doc = include_str!("material_theme.doc/_module.md")]

#[doc = include_str!("material_theme.doc/MATERIAL_CSS.md")]
pub(super) const MATERIAL_CSS: &str = r"
* { box-sizing: border-box; }
#app { display: flex; flex-direction: column; background-color: #FEF7FF; }

/* ---- Switcher bar ---- */
.kit-switcher { display: flex; flex-direction: row; align-items: center; height: 60px;
    padding-left: 18px; padding-right: 18px; background-color: #16181f;
    border-bottom: 1px solid #262a36; }
.kit-brand { display: flex; flex-direction: row; align-items: center; width: 60px; height: 60px; }
.kit-logo { width: 11px; height: 11px; border-radius: 3px; background-color: #e8eaf0; }
.kit-brand-name { width: 40px; height: 16px; margin-left: 8px; color: #e8eaf0;
    font-size: 13px; font-weight: 700; }
.kit-tabs { display: flex; flex-direction: row; }
.kit-tab { display: flex; flex-direction: column; align-items: center; justify-content: center;
    width: 108px; height: 60px; }
.kit-tab-label { height: 16px; color: #9aa0b0; font-size: 12px; font-weight: 500; text-align: center; }
.kit-tab.active .kit-tab-label { color: #e8eaf0; }
.kit-underline { width: 60px; height: 2px; border-radius: 2px; background-color: #e8eaf0; margin-top: 6px; }

/* ---- Gallery frame ---- */
.m3-page { display: flex; flex-direction: column; padding: 34px 40px 60px; }
.m3-head { display: flex; flex-direction: column; height: 108px; margin-bottom: 26px; }
.m3-eyebrow { height: 16px; color: #7D7787; font-size: 11px; font-weight: 500; }
.m3-title { height: 48px; color: #1D1B20; font-size: 38px; font-weight: 400; }
.m3-sub { height: 22px; color: #49454F; font-size: 15px; }

.gallery-grid { display: flex; flex-direction: column; }
.row { display: flex; flex-direction: row; align-items: stretch; margin-bottom: 24px; }
.row.r1 { height: 232px; }
.row.r2 { height: 336px; }
.row.r3 { height: 420px; }
.row.r4 { height: 252px; }

.card { background-color: #ffffff; border-radius: 16px; padding: 22px;
    box-shadow: 0 1px 3px rgba(0,0,0,0.10), 0 1px 2px rgba(0,0,0,0.06); }
.card.col { width: 588px; margin-right: 24px; }
.card.full { width: 1200px; margin-bottom: 24px; }
.card.full.table-c { height: 288px; }
.card.full.feedback-c { height: 132px; }
.card-label { height: 16px; margin-bottom: 16px; color: #7D7787; font-size: 11px; font-weight: 500; }

/* ---- Buttons card ---- */
.btn-row { display: flex; flex-direction: row; align-items: center; height: 56px; }
.btn { display: flex; flex-direction: row; align-items: center; justify-content: center;
    height: 40px; border-radius: 100px; padding-left: 22px; padding-right: 22px; margin-right: 10px; }
.btn-label { height: 18px; font-size: 14px; font-weight: 500; text-align: center; }
.btn-filled { width: 110px; background-color: #6750A4; }
.btn-filled .btn-label { color: #ffffff; }
.btn-tonal { width: 104px; background-color: #E8DEF8; }
.btn-tonal .btn-label { color: #1D192B; }
.btn-outlined { width: 128px; background-color: #FEF7FF; border: 1px solid #79747E; }
.btn-outlined .btn-label { color: #6750A4; }
.btn-text { width: 92px; padding-left: 16px; padding-right: 16px; background-color: #FEF7FF; }
.btn-text .btn-label { color: #6750A4; }
.btn-dialog { width: 152px; }
.btn-elevated { width: 132px; background-color: #F7F2FA;
    box-shadow: 0 1px 3px rgba(0,0,0,0.18); }
.btn-elevated .btn-label { color: #6750A4; }
.btn-disabled { width: 128px; background-color: rgba(29,27,32,0.12); }
.btn-disabled .btn-label { color: rgba(29,27,32,0.38); }
.fab { display: flex; flex-direction: row; align-items: center; justify-content: center;
    width: 56px; height: 56px; border-radius: 18px; background-color: #EADDFF;
    box-shadow: 0 1px 3px rgba(0,0,0,0.20), 0 4px 8px rgba(0,0,0,0.12); }
.fab-glyph { height: 30px; color: #21005D; font-size: 26px; font-weight: 400; text-align: center; }
.btn-sm { height: 38px; }
.btn-wide { width: 168px; }

/* ---- Selection card ---- */
.sel-cols { display: flex; flex-direction: row; }
.sel-col { display: flex; flex-direction: column; width: 168px; }
.sel-item { display: flex; flex-direction: row; align-items: center; height: 34px; }
.sel-col .sel-item { margin-bottom: 14px; }
.sel-col.wide-gap .sel-item { margin-bottom: 18px; height: 32px; }
.sel-text { height: 20px; margin-left: 11px; color: #1D1B20; font-size: 14px; }
.checkbox-on { display: flex; align-items: center; justify-content: center;
    width: 18px; height: 18px; border-radius: 3px; background-color: #6750A4; }
.check-glyph { height: 14px; color: #ffffff; font-size: 12px; text-align: center; }
.checkbox-off { width: 18px; height: 18px; border-radius: 3px; border: 2px solid #49454F; }
.checkbox-disabled { width: 18px; height: 18px; border-radius: 3px; border: 2px solid rgba(29,27,32,0.20); }
.checkbox-disabled ~ .sel-text, .sel-item .checkbox-disabled + .sel-text { color: rgba(29,27,32,0.38); }
.radio-on { display: flex; align-items: center; justify-content: center;
    width: 20px; height: 20px; border-radius: 10px; border: 2px solid #6750A4; }
.radio-dot { width: 10px; height: 10px; border-radius: 5px; background-color: #6750A4; }
.radio-off { width: 20px; height: 20px; border-radius: 10px; border: 2px solid #79747E; }
.toggle-on { display: flex; flex-direction: row; align-items: center; justify-content: flex-end;
    width: 52px; height: 32px; border-radius: 16px; background-color: #6750A4; padding: 4px; }
.toggle-knob { width: 24px; height: 24px; border-radius: 12px; background-color: #ffffff; }
.toggle-off { display: flex; flex-direction: row; align-items: center; justify-content: flex-start;
    width: 52px; height: 32px; border-radius: 16px; background-color: #E6E0E9;
    border: 2px solid #79747E; padding: 6px; }
.toggle-knob-off { width: 16px; height: 16px; border-radius: 8px; background-color: #79747E; }

/* ---- Text fields card ---- */
.field-col { display: flex; flex-direction: column; }
.field { margin-bottom: 16px; }
.field-filled { background-color: #E7E0EC; border-radius: 4px 4px 0 0;
    border-bottom: 2px solid #6750A4; padding: 8px 14px 6px; }
.field-outlined { border: 1px solid #79747E; border-radius: 4px; padding: 10px 14px; }
.field-error { border: 2px solid #B3261E; border-radius: 4px; padding: 10px 14px; }
.field-label-primary { height: 14px; margin-bottom: 4px; color: #6750A4; font-size: 11px; }
.field-label { height: 14px; margin-bottom: 4px; color: #49454F; font-size: 11px; }
.field-label-error { height: 14px; margin-bottom: 4px; color: #B3261E; font-size: 11px; }
.field-value { height: 20px; color: #1D1B20; font-size: 15px; }
.field-placeholder { height: 20px; color: #79747E; font-size: 15px; }
.field-error-text { height: 14px; margin-top: 5px; color: #B3261E; font-size: 11px; }

/* ---- Menu card ---- */
.select-box { display: flex; flex-direction: row; align-items: center; justify-content: space-between;
    width: 260px; height: 44px; border: 1px solid #79747E; border-radius: 4px; padding: 0 14px; }
.select-value { height: 20px; color: #1D1B20; font-size: 15px; }
.select-chevron { width: 16px; height: 20px; color: #49454F; font-size: 14px; text-align: right; }
.menu-panel { display: flex; flex-direction: column; width: 260px; height: 136px; margin-top: 8px;
    background-color: #F7F2FA; border-radius: 8px; padding-top: 8px; padding-bottom: 8px;
    box-shadow: 0 4px 12px rgba(0,0,0,0.18), 0 1px 3px rgba(0,0,0,0.12); }
.menu-item { display: flex; flex-direction: row; align-items: center; justify-content: space-between;
    height: 40px; padding: 0 16px; }
.menu-item.selected { background-color: #EADDFF; }
.menu-text { height: 20px; color: #1D1B20; font-size: 15px; }
.menu-check { width: 16px; height: 20px; color: #6750A4; font-size: 14px; text-align: right; }
.chips-row { display: flex; flex-direction: row; align-items: center; height: 32px; margin-top: 16px; }
.chip { display: flex; flex-direction: row; align-items: center; justify-content: center; height: 32px;
    border-radius: 8px; padding-left: 14px; padding-right: 14px; margin-right: 8px; }
.chip-filled { width: 112px; background-color: #E8DEF8; }
.chip-filled .chip-text { color: #1D192B; }
.chip-check { height: 16px; margin-right: 6px; color: #1D192B; font-size: 12px; }
.chip-outlined { width: 104px; border: 1px solid #79747E; }
.chip-outlined .chip-text { color: #49454F; }
.chip-tonal { background-color: #E8DEF8; }
.chip-tonal .chip-text { color: #1D192B; }
.chip-wide { width: 172px; }
.chip-text { height: 18px; font-size: 14px; font-weight: 500; }

/* ---- Card & list ---- */
.mini-card { display: flex; flex-direction: column; height: 208px; background-color: #F7F2FA;
    border-radius: 12px; box-shadow: 0 1px 3px rgba(0,0,0,0.12); }
.mini-cover { display: flex; align-items: center; justify-content: center; height: 84px;
    border-radius: 12px 12px 0 0;
    background-image: linear-gradient(135deg, #E8DEF8, #EADDFF); }
.mini-cover-text { height: 16px; color: #6750A4; font-size: 11px; text-align: center; }
.mini-body { display: flex; flex-direction: column; height: 124px; padding: 16px; }
.mini-title { height: 22px; color: #1D1B20; font-size: 16px; font-weight: 500; }
.mini-sub { height: 18px; margin-top: 4px; color: #49454F; font-size: 13px; }
.mini-actions { display: flex; flex-direction: row; align-items: center; height: 38px; margin-top: 14px; }
.people { display: flex; flex-direction: column; height: 100px; margin-top: 14px; }
.person { display: flex; flex-direction: row; align-items: center; height: 50px; }
.person-meta { display: flex; flex-direction: column; justify-content: center; height: 50px;
    margin-left: 13px; width: 420px; }
.person-name { height: 18px; color: #1D1B20; font-size: 14px; }
.person-email { height: 16px; margin-top: 2px; color: #49454F; font-size: 12px; }
.person-menu { width: 24px; height: 20px; color: #79747E; font-size: 18px; text-align: right; }
.avatar { display: flex; align-items: center; justify-content: center;
    width: 36px; height: 36px; border-radius: 18px; }
.avatar-text { height: 16px; font-size: 13px; font-weight: 500; text-align: center; }
.avatar-ak { background-color: #6750A4; }
.avatar-ak .avatar-text { color: #ffffff; }
.avatar-mr { background-color: #7D5260; }
.avatar-mr .avatar-text { color: #ffffff; }
.avatar-jp { background-color: #386641; }
.avatar-jp .avatar-text { color: #ffffff; }
.avatar-more { background-color: #E8DEF8; }
.avatar-more .avatar-text { color: #6750A4; }

/* ---- Navigation ---- */
.breadcrumb { display: flex; flex-direction: row; align-items: center; height: 20px; margin-bottom: 18px; }
.crumb { height: 18px; margin-right: 8px; color: #49454F; font-size: 13px; }
.crumb-sep { height: 18px; margin-right: 8px; color: #CAC4D0; font-size: 13px; }
.crumb-active { height: 18px; color: #6750A4; font-size: 13px; font-weight: 500; }
.nav-tabs { display: flex; flex-direction: row; height: 42px;
    border-bottom: 1px solid #E7E0EC; margin-bottom: 14px; }
.nav-tab { display: flex; flex-direction: column; align-items: center; justify-content: flex-end;
    width: 96px; }
.nav-tab-label { height: 24px; color: #49454F; font-size: 14px; font-weight: 500; text-align: center; }
.nav-tab-label.active { color: #6750A4; }
.nav-tab-underline { width: 72px; height: 3px; border-radius: 3px 3px 0 0; background-color: #6750A4; }
.nav-list { display: flex; flex-direction: column; width: 260px; }
.nav-item { display: flex; flex-direction: row; align-items: center; height: 44px;
    border-radius: 100px; padding-left: 16px; padding-right: 16px; margin-bottom: 4px; }
.nav-item.active { background-color: #E8DEF8; }
.nav-item-text { height: 20px; margin-left: 12px; color: #49454F; font-size: 14px; }
.nav-item.active .nav-item-text { color: #1D192B; font-weight: 500; }
.dot-on { width: 8px; height: 8px; border-radius: 4px; background-color: #6750A4; }
.dot-off { width: 8px; height: 8px; border-radius: 4px; background-color: #CAC4D0; }

/* ---- Badges / chips / avatars ---- */
.badge-row { display: flex; flex-direction: row; align-items: center; height: 26px; margin-bottom: 22px; }
.badge { display: flex; align-items: center; justify-content: center; height: 26px;
    border-radius: 8px; margin-right: 8px; }
.badge-text { height: 16px; font-size: 12px; font-weight: 500; text-align: center; }
.badge-success { width: 76px; background-color: #D5F2DF; }
.badge-success .badge-text { color: #0F5132; }
.badge-warn { width: 84px; background-color: #FCE8C9; }
.badge-warn .badge-text { color: #7A4F01; }
.badge-error { width: 66px; background-color: #F9DEDC; }
.badge-error .badge-text { color: #B3261E; }
.badge-neutral { width: 64px; background-color: #E7E0EC; }
.badge-neutral .badge-text { color: #49454F; }
.badge-count { display: flex; align-items: center; justify-content: center;
    width: 24px; height: 20px; border-radius: 10px; background-color: #B3261E; }
.badge-count-text { height: 14px; color: #ffffff; font-size: 11px; font-weight: 700; text-align: center; }
.avatar-stack { display: flex; flex-direction: row; align-items: center; height: 40px; }
.avatar.stack { border: 2px solid #ffffff; }
.avatar.overlap { margin-left: -10px; }

/* ---- Sliders / progress ---- */
.slider-head { display: flex; flex-direction: row; align-items: center; justify-content: space-between;
    height: 18px; margin-bottom: 10px; }
.slider-name { height: 16px; color: #49454F; font-size: 13px; }
.slider-val { height: 16px; color: #6750A4; font-size: 13px; text-align: right; }
.slider { position: relative; height: 24px; margin-bottom: 24px; }
.slider-track { position: absolute; left: 0; right: 0; top: 10px; height: 4px;
    border-radius: 2px; background-color: #E7E0EC; }
.slider-fill { position: absolute; left: 0; top: 10px; width: 55%; height: 4px;
    border-radius: 2px; background-color: #6750A4; }
.slider-knob { position: absolute; left: 53%; top: 2px; width: 20px; height: 20px;
    border-radius: 10px; background-color: #6750A4; }
.progress-label { height: 18px; margin-bottom: 8px; color: #49454F; font-size: 13px; }
.progress { height: 4px; border-radius: 2px; background-color: #E7E0EC; margin-bottom: 20px; }
.progress-fill { width: 72%; height: 4px; border-radius: 2px; background-color: #6750A4; }
.spinner-row { display: flex; flex-direction: row; align-items: center; height: 28px; }
.spinner { width: 28px; height: 28px; border-radius: 14px; margin-right: 14px;
    border: 3px solid #E7E0EC; border-top-color: #6750A4; }
.spinner-text { height: 18px; color: #49454F; font-size: 13px; }

/* ---- Data table ---- */
.table { border: 1px solid #E7E0EC; border-radius: 12px; }
.thead { display: grid; grid-template-columns: 1.6fr 1fr 1fr 0.8fr;
    background-color: #F7F2FA; padding: 14px 18px; border-radius: 12px 12px 0 0; }
.th { height: 16px; color: #49454F; font-size: 12px; font-weight: 500; }
.th-right { text-align: right; }
.trow { display: grid; grid-template-columns: 1.6fr 1fr 1fr 0.8fr; align-items: center;
    padding: 14px 18px; border-top: 1px solid #E7E0EC; }
.td { height: 20px; color: #1D1B20; font-size: 14px; }
.td-name { font-weight: 500; }
.td-muted { color: #49454F; }
.td-mono { color: #1D1B20; }
.td-right { text-align: right; }

/* ---- Feedback ---- */
.feedback-row { display: flex; flex-direction: row; align-items: center; height: 40px; }
";
