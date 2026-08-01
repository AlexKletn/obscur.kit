#![doc = include_str!("minimal_theme.doc/_module.md")]

#[doc = include_str!("minimal_theme.doc/MINIMAL_CSS.md")]
pub(super) const MINIMAL_CSS: &str = r"
* { box-sizing: border-box; }
#app { display: flex; flex-direction: column; background-color: #f6f6f3; }

/* ---- Switcher bar: light + airy (Minimal inverts Material's dark strip) ---- */
.kit-switcher { display: flex; flex-direction: row; align-items: center; height: 60px;
    padding-left: 18px; padding-right: 18px; background-color: #ffffff;
    border-bottom: 1px solid #e7e7e1; }
.kit-brand { display: flex; flex-direction: row; align-items: center; width: 60px; height: 60px; }
.kit-logo { width: 11px; height: 11px; border-radius: 4px; background-color: #407cc5; }
.kit-brand-name { width: 40px; height: 16px; margin-left: 8px; color: #1b1b18;
    font-size: 13px; font-weight: 700; }
.kit-tabs { display: flex; flex-direction: row; }
.kit-tab { display: flex; flex-direction: column; align-items: center; justify-content: center;
    width: 108px; height: 60px; }
.kit-tab-label { height: 16px; color: #9a9a92; font-size: 12px; font-weight: 500; text-align: center; }
.kit-tab.active .kit-tab-label { color: #1b1b18; }
.kit-underline { width: 60px; height: 2px; border-radius: 2px; background-color: #407cc5; margin-top: 6px; }

/* ---- Gallery frame ---- */
.m3-page { display: flex; flex-direction: column; padding: 34px 40px 60px; }
.m3-head { display: flex; flex-direction: column; height: 108px; margin-bottom: 26px; }
.m3-eyebrow { height: 16px; color: #a3a399; font-size: 11px; font-weight: 500; }
.m3-title { height: 48px; color: #1b1b18; font-size: 38px; font-weight: 600; }
.m3-sub { height: 22px; color: #76766d; font-size: 15px; }

.gallery-grid { display: flex; flex-direction: column; }
.row { display: flex; flex-direction: row; align-items: stretch; margin-bottom: 24px; }
.row.r1 { height: 232px; }
.row.r2 { height: 336px; }
.row.r3 { height: 420px; }
.row.r4 { height: 252px; }

/* Cards: white with ONE hairline border, small radius, no shadow. */
.card { background-color: #ffffff; border-radius: 14px; padding: 22px;
    border: 1px solid #e7e7e1; }
.card.col { width: 588px; margin-right: 24px; }
.card.full { width: 1200px; margin-bottom: 24px; }
.card.full.table-c { height: 288px; }
.card.full.feedback-c { height: 132px; }
.card-label { height: 16px; margin-bottom: 16px; color: #a3a399; font-size: 11px; font-weight: 500; }

/* ---- Buttons: 8px rounded rects, one accent, otherwise white+border ---- */
.btn-row { display: flex; flex-direction: row; align-items: center; height: 56px; }
.btn { display: flex; flex-direction: row; align-items: center; justify-content: center;
    height: 40px; border-radius: 8px; padding-left: 20px; padding-right: 20px; margin-right: 10px; }
.btn-label { height: 18px; font-size: 14px; font-weight: 500; text-align: center; }
.btn-filled { width: 110px; background-color: #407cc5; }
.btn-filled .btn-label { color: #ffffff; }
.btn-tonal { width: 104px; background-color: #f3f3ee; border: 1px solid #e7e7e1; }
.btn-tonal .btn-label { color: #26261f; }
.btn-outlined { width: 128px; background-color: #ffffff; border: 1px solid #d9d9d1; }
.btn-outlined .btn-label { color: #26261f; }
.btn-text { width: 92px; padding-left: 16px; padding-right: 16px; background-color: #ffffff; }
.btn-text .btn-label { color: #3568b0; }
.btn-dialog { width: 152px; }
.btn-elevated { width: 132px; background-color: #ffffff; border: 1px solid #e7e7e1;
    box-shadow: 0 1px 2px rgba(20,20,16,0.06); }
.btn-elevated .btn-label { color: #26261f; }
.btn-disabled { width: 128px; background-color: #f3f3ee; border: 1px solid #e7e7e1; }
.btn-disabled .btn-label { color: #bcbcb2; }
/* FAB → Minimal has no floating action button; render the icon-square affordance. */
.fab { display: flex; flex-direction: row; align-items: center; justify-content: center;
    width: 56px; height: 56px; border-radius: 8px; background-color: #ffffff; border: 1px solid #d9d9d1; }
.fab-glyph { height: 30px; color: #5a5a52; font-size: 26px; font-weight: 400; text-align: center; }
.btn-sm { height: 38px; }
.btn-wide { width: 168px; }

/* ---- Selection ---- */
.sel-cols { display: flex; flex-direction: row; }
.sel-col { display: flex; flex-direction: column; width: 168px; }
.sel-item { display: flex; flex-direction: row; align-items: center; height: 34px; }
.sel-col .sel-item { margin-bottom: 14px; }
.sel-col.wide-gap .sel-item { margin-bottom: 18px; height: 32px; }
.sel-text { height: 20px; margin-left: 11px; color: #1b1b18; font-size: 14px; }
.checkbox-on { display: flex; align-items: center; justify-content: center;
    width: 18px; height: 18px; border-radius: 6px; background-color: #407cc5; }
.check-glyph { height: 14px; color: #ffffff; font-size: 12px; text-align: center; }
.checkbox-off { width: 18px; height: 18px; border-radius: 6px; border: 1px solid #cccbc1; }
.checkbox-disabled { width: 18px; height: 18px; border-radius: 6px; border: 1px solid #e4e4de;
    background-color: #f3f3ee; }
.radio-on { display: flex; align-items: center; justify-content: center;
    width: 20px; height: 20px; border-radius: 10px; border: 1px solid #cccbc1; }
.radio-dot { width: 10px; height: 10px; border-radius: 5px; background-color: #407cc5; }
.radio-off { width: 20px; height: 20px; border-radius: 10px; border: 1px solid #cccbc1; }
.toggle-on { display: flex; flex-direction: row; align-items: center; justify-content: flex-end;
    width: 44px; height: 24px; border-radius: 12px; background-color: #407cc5; padding: 3px; }
.toggle-knob { width: 18px; height: 18px; border-radius: 9px; background-color: #ffffff;
    box-shadow: 0 1px 2px rgba(0,0,0,0.25); }
.toggle-off { display: flex; flex-direction: row; align-items: center; justify-content: flex-start;
    width: 44px; height: 24px; border-radius: 12px; background-color: #d6d6cd; padding: 3px; }
.toggle-knob-off { width: 18px; height: 18px; border-radius: 9px; background-color: #ffffff;
    box-shadow: 0 1px 2px rgba(0,0,0,0.25); }

/* ---- Text fields: soft off-white wells, hairline border, 8px radius ---- */
.field-col { display: flex; flex-direction: column; }
.field { margin-bottom: 16px; }
.field-filled { background-color: #fafaf7; border-radius: 8px; border: 1px solid #d9d9d1; padding: 9px 12px; }
.field-outlined { background-color: #fafaf7; border: 1px solid #d9d9d1; border-radius: 8px; padding: 9px 12px; }
.field-error { background-color: #fdf4f3; border: 1px solid #d97064; border-radius: 8px; padding: 9px 12px; }
.field-label-primary { height: 14px; margin-bottom: 4px; color: #76766d; font-size: 11px; }
.field-label { height: 14px; margin-bottom: 4px; color: #76766d; font-size: 11px; }
.field-label-error { height: 14px; margin-bottom: 4px; color: #c0392b; font-size: 11px; }
.field-value { height: 20px; color: #1b1b18; font-size: 15px; }
.field-placeholder { height: 20px; color: #a3a399; font-size: 15px; }
.field-error-text { height: 14px; margin-top: 5px; color: #c0392b; font-size: 11px; }

/* ---- Menu / dropdown ---- */
.select-box { display: flex; flex-direction: row; align-items: center; justify-content: space-between;
    width: 260px; height: 44px; background-color: #fafaf7; border: 1px solid #d9d9d1; border-radius: 8px; padding: 0 14px; }
.select-value { height: 20px; color: #1b1b18; font-size: 15px; }
.select-chevron { width: 16px; height: 20px; color: #a3a399; font-size: 14px; text-align: right; }
.menu-panel { display: flex; flex-direction: column; width: 260px; height: 136px; margin-top: 8px;
    background-color: #ffffff; border: 1px solid #e4e4de; border-radius: 8px; padding-top: 6px; padding-bottom: 6px;
    box-shadow: 0 8px 24px rgba(20,20,16,0.10); }
.menu-item { display: flex; flex-direction: row; align-items: center; justify-content: space-between;
    height: 40px; padding: 0 14px; }
.menu-item.selected { background-color: #f3f3ee; border-radius: 6px; }
.menu-text { height: 20px; color: #1b1b18; font-size: 15px; }
.menu-check { width: 16px; height: 20px; color: #407cc5; font-size: 14px; text-align: right; }
.chips-row { display: flex; flex-direction: row; align-items: center; height: 32px; margin-top: 16px; }
.chip { display: flex; flex-direction: row; align-items: center; justify-content: center; height: 32px;
    border-radius: 8px; padding-left: 12px; padding-right: 12px; margin-right: 8px; }
.chip-filled { width: 112px; background-color: #f3f3ee; border: 1px solid #e4e4de; }
.chip-filled .chip-text { color: #26261f; }
.chip-check { height: 16px; margin-right: 6px; color: #76766d; font-size: 12px; }
.chip-outlined { width: 104px; background-color: #f3f3ee; border: 1px solid #e4e4de; }
.chip-outlined .chip-text { color: #26261f; }
.chip-tonal { background-color: #f3f3ee; border: 1px solid #e4e4de; }
.chip-tonal .chip-text { color: #26261f; }
.chip-wide { width: 172px; }
.chip-text { height: 18px; font-size: 14px; font-weight: 500; }

/* ---- Card & list ---- */
.mini-card { display: flex; flex-direction: column; height: 208px; background-color: #ffffff;
    border: 1px solid #ebebe4; border-radius: 12px; }
.mini-cover { display: flex; align-items: center; justify-content: center; height: 84px;
    border-radius: 12px 12px 0 0; background-color: #eeeee7; }
.mini-cover-text { height: 16px; color: #b6b6ac; font-size: 11px; text-align: center; }
.mini-body { display: flex; flex-direction: column; height: 124px; padding: 16px; }
.mini-title { height: 22px; color: #1b1b18; font-size: 16px; font-weight: 600; }
.mini-sub { height: 18px; margin-top: 4px; color: #76766d; font-size: 13px; }
.mini-actions { display: flex; flex-direction: row; align-items: center; height: 38px; margin-top: 14px; }
.people { display: flex; flex-direction: column; height: 100px; margin-top: 14px; }
.person { display: flex; flex-direction: row; align-items: center; height: 50px; }
.person-meta { display: flex; flex-direction: column; justify-content: center; height: 50px;
    margin-left: 13px; width: 420px; }
.person-name { height: 18px; color: #1b1b18; font-size: 14px; }
.person-email { height: 16px; margin-top: 2px; color: #76766d; font-size: 12px; }
.person-menu { width: 24px; height: 20px; color: #a3a399; font-size: 18px; text-align: right; }
.avatar { display: flex; align-items: center; justify-content: center;
    width: 36px; height: 36px; border-radius: 18px; }
.avatar-text { height: 16px; font-size: 13px; font-weight: 600; text-align: center; color: #ffffff; }
.avatar-ak { background-color: #d58679; }
.avatar-mr { background-color: #6fb07d; }
.avatar-jp { background-color: #7a9fdd; }
.avatar-more { background-color: #e9e9e2; }
.avatar-more .avatar-text { color: #76766d; }

/* ---- Navigation ---- */
.breadcrumb { display: flex; flex-direction: row; align-items: center; height: 20px; margin-bottom: 18px; }
.crumb { height: 18px; margin-right: 8px; color: #76766d; font-size: 13px; }
.crumb-sep { height: 18px; margin-right: 8px; color: #cccbc1; font-size: 13px; }
.crumb-active { height: 18px; color: #1b1b18; font-size: 13px; font-weight: 500; }
.nav-tabs { display: flex; flex-direction: row; height: 42px;
    border-bottom: 1px solid #ebebe4; margin-bottom: 14px; }
.nav-tab { display: flex; flex-direction: column; align-items: center; justify-content: flex-end;
    width: 96px; }
.nav-tab-label { height: 24px; color: #9a9a90; font-size: 14px; font-weight: 500; text-align: center; }
.nav-tab-label.active { color: #1b1b18; }
.nav-tab-underline { width: 72px; height: 2px; border-radius: 2px; background-color: #407cc5; }
.nav-list { display: flex; flex-direction: column; width: 260px; }
.nav-item { display: flex; flex-direction: row; align-items: center; height: 40px;
    border-radius: 7px; padding-left: 12px; padding-right: 12px; margin-bottom: 4px; }
.nav-item.active { background-color: #eaf0f9; }
.nav-item-text { height: 20px; margin-left: 10px; color: #5a5a52; font-size: 14px; }
.nav-item.active .nav-item-text { color: #2b5697; font-weight: 500; }
.dot-on { width: 6px; height: 6px; border-radius: 3px; background-color: #407cc5; }
.dot-off { width: 6px; height: 6px; border-radius: 3px; background-color: #cccbc1; }

/* ---- Badges / chips / avatars: muted pastel pills ---- */
.badge-row { display: flex; flex-direction: row; align-items: center; height: 26px; margin-bottom: 22px; }
.badge { display: flex; align-items: center; justify-content: center; height: 24px;
    border-radius: 20px; margin-right: 8px; }
.badge-text { height: 16px; font-size: 12px; font-weight: 500; text-align: center; }
.badge-success { width: 76px; background-color: #e6f4ea; }
.badge-success .badge-text { color: #0f6634; }
.badge-warn { width: 84px; background-color: #f3eeda; }
.badge-warn .badge-text { color: #7a4f01; }
.badge-error { width: 66px; background-color: #f7e9e5; }
.badge-error .badge-text { color: #a83634; }
.badge-neutral { width: 64px; background-color: #efefe8; }
.badge-neutral .badge-text { color: #76766d; }
.badge-count { display: flex; align-items: center; justify-content: center;
    width: 24px; height: 20px; border-radius: 6px; background-color: #407cc5; }
.badge-count-text { height: 14px; color: #ffffff; font-size: 11px; font-weight: 700; text-align: center; }
.avatar-stack { display: flex; flex-direction: row; align-items: center; height: 40px; }
.avatar.stack { border: 2px solid #ffffff; }
.avatar.overlap { margin-left: -9px; }

/* ---- Sliders / progress ---- */
.slider-head { display: flex; flex-direction: row; align-items: center; justify-content: space-between;
    height: 18px; margin-bottom: 10px; }
.slider-name { height: 16px; color: #76766d; font-size: 13px; }
.slider-val { height: 16px; color: #1b1b18; font-size: 13px; text-align: right; }
.slider { position: relative; height: 24px; margin-bottom: 24px; }
.slider-track { position: absolute; left: 0; right: 0; top: 10px; height: 5px;
    border-radius: 3px; background-color: #e4e4de; }
.slider-fill { position: absolute; left: 0; top: 10px; width: 55%; height: 5px;
    border-radius: 3px; background-color: #407cc5; }
.slider-knob { position: absolute; left: 53%; top: 4px; width: 17px; height: 17px;
    border-radius: 9px; background-color: #ffffff; border: 1px solid #cccbc1;
    box-shadow: 0 1px 3px rgba(0,0,0,0.18); }
.progress-label { height: 18px; margin-bottom: 8px; color: #76766d; font-size: 13px; }
.progress { height: 8px; border-radius: 4px; background-color: #e4e4de; margin-bottom: 20px; }
.progress-fill { width: 72%; height: 8px; border-radius: 4px; background-color: #407cc5; }
.spinner-row { display: flex; flex-direction: row; align-items: center; height: 28px; }
.spinner { width: 24px; height: 24px; border-radius: 12px; margin-right: 14px;
    border: 3px solid #e4e4de; border-top-color: #407cc5; }
.spinner-text { height: 18px; color: #76766d; font-size: 13px; }

/* ---- Data table ---- */
.table { border: 1px solid #ebebe4; border-radius: 10px; }
.thead { display: grid; grid-template-columns: 1.6fr 1fr 1fr 0.8fr;
    background-color: #fafaf7; padding: 12px 16px; border-radius: 10px 10px 0 0; }
.th { height: 16px; color: #a3a399; font-size: 12px; font-weight: 600; }
.th-right { text-align: right; }
.trow { display: grid; grid-template-columns: 1.6fr 1fr 1fr 0.8fr; align-items: center;
    padding: 12px 16px; border-top: 1px solid #f0f0e9; }
.td { height: 20px; color: #1b1b18; font-size: 14px; }
.td-name { font-weight: 500; }
.td-muted { color: #76766d; }
.td-mono { color: #1b1b18; }
.td-right { text-align: right; }

/* ---- Feedback ---- */
.feedback-row { display: flex; flex-direction: row; align-items: center; height: 40px; }
";
