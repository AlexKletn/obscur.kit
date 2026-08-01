#![doc = include_str!("metro_theme.doc/_module.md")]

#[doc = include_str!("metro_theme.doc/METRO_CSS.md")]
pub(super) const METRO_CSS: &str = r"
* { box-sizing: border-box; }
#app { display: flex; flex-direction: column; background-color: #ffffff; }

/* ---- Switcher bar ---- */
.kit-switcher { display: flex; flex-direction: row; align-items: center; height: 60px;
    padding-left: 18px; padding-right: 18px; background-color: #1d1d1d; }
.kit-brand { display: flex; flex-direction: row; align-items: center; width: 60px; height: 60px; }
.kit-logo { width: 11px; height: 11px; background-color: #2d89ef; }
.kit-brand-name { width: 40px; height: 16px; margin-left: 8px; color: #ffffff;
    font-size: 13px; font-weight: 600; }
.kit-tabs { display: flex; flex-direction: row; }
.kit-tab { display: flex; flex-direction: column; align-items: center; justify-content: center;
    width: 108px; height: 60px; }
.kit-tab-label { height: 16px; color: #9a9a9a; font-size: 12px; font-weight: 400; text-align: center; }
.kit-tab.active .kit-tab-label { color: #ffffff; }
.kit-underline { width: 60px; height: 3px; background-color: #2d89ef; margin-top: 6px; }

/* ---- Gallery frame ---- */
.m3-page { display: flex; flex-direction: column; padding: 34px 40px 60px; }
.m3-head { display: flex; flex-direction: column; height: 108px; margin-bottom: 26px; }
.m3-eyebrow { height: 16px; color: #9a9a9a; font-size: 11px; font-weight: 400; }
.m3-title { height: 48px; color: #1d1d1d; font-size: 40px; font-weight: 300; }
.m3-sub { height: 22px; color: #6a6a6a; font-size: 15px; }

.gallery-grid { display: flex; flex-direction: column; }
.row { display: flex; flex-direction: row; align-items: stretch; margin-bottom: 20px; }
.row.r1 { height: 232px; }
.row.r2 { height: 336px; }
.row.r3 { height: 420px; }
.row.r4 { height: 252px; }

/* Cards: flat gray tiles — no border, no shadow, ZERO radius. */
.card { background-color: #f2f2f2; border-radius: 0; padding: 22px; }
.card.col { width: 590px; margin-right: 20px; }
.card.full { width: 1200px; margin-bottom: 20px; }
.card.full.table-c { height: 288px; }
.card.full.feedback-c { height: 132px; }
.card-label { height: 16px; margin-bottom: 16px; color: #9a9a9a; font-size: 11px; font-weight: 400; }

/* ---- Buttons: flat sharp blocks ---- */
.btn-row { display: flex; flex-direction: row; align-items: center; height: 56px; }
.btn { display: flex; flex-direction: row; align-items: center; justify-content: center;
    height: 40px; border-radius: 0; padding-left: 20px; padding-right: 20px; margin-right: 10px; }
.btn-label { height: 18px; font-size: 14px; font-weight: 600; text-align: center; }
.btn-filled { width: 110px; background-color: #2d89ef; }
.btn-filled .btn-label { color: #ffffff; }
.btn-tonal { width: 104px; background-color: #ffffff; border: 2px solid #1d1d1d; }
.btn-tonal .btn-label { color: #1d1d1d; }
.btn-outlined { width: 128px; background-color: #ffffff; border: 2px solid #1d1d1d; }
.btn-outlined .btn-label { color: #1d1d1d; }
.btn-text { width: 92px; padding-left: 16px; padding-right: 16px; background-color: #f2f2f2; }
.btn-text .btn-label { color: #2d89ef; }
.btn-dialog { width: 152px; }
.btn-elevated { width: 132px; background-color: #da532c; }
.btn-elevated .btn-label { color: #ffffff; }
.btn-disabled { width: 128px; background-color: #d6d6d6; }
.btn-disabled .btn-label { color: #9a9a9a; }
/* FAB → a flat square accent tile with the + glyph. */
.fab { display: flex; flex-direction: row; align-items: center; justify-content: center;
    width: 56px; height: 56px; border-radius: 0; background-color: #ffffff; border: 2px solid #d0d0d0; }
.fab-glyph { height: 30px; color: #2d89ef; font-size: 26px; font-weight: 400; text-align: center; }
.btn-sm { height: 38px; }
.btn-wide { width: 168px; }

/* ---- Selection: sharp square checks, round radios, rectangular toggles ---- */
.sel-cols { display: flex; flex-direction: row; }
.sel-col { display: flex; flex-direction: column; width: 168px; }
.sel-item { display: flex; flex-direction: row; align-items: center; height: 34px; }
.sel-col .sel-item { margin-bottom: 14px; }
.sel-col.wide-gap .sel-item { margin-bottom: 18px; height: 32px; }
.sel-text { height: 20px; margin-left: 11px; color: #1d1d1d; font-size: 14px; }
.checkbox-on { display: flex; align-items: center; justify-content: center;
    width: 20px; height: 20px; border-radius: 0; background-color: #2d89ef; }
.check-glyph { height: 14px; color: #ffffff; font-size: 12px; text-align: center; }
.checkbox-off { width: 20px; height: 20px; border-radius: 0; background-color: #ffffff; border: 2px solid #5a5a5a; }
.checkbox-disabled { width: 20px; height: 20px; border-radius: 0; background-color: #e6e6e6; border: 2px solid #cfcfcf; }
/* Radio is the one deliberately round Metro control. */
.radio-on { display: flex; align-items: center; justify-content: center;
    width: 20px; height: 20px; border-radius: 10px; background-color: #ffffff; border: 2px solid #5a5a5a; }
.radio-dot { width: 10px; height: 10px; border-radius: 5px; background-color: #2d89ef; }
.radio-off { width: 20px; height: 20px; border-radius: 10px; background-color: #ffffff; border: 2px solid #5a5a5a; }
/* Toggle: a rectangular bordered box with a sliding rectangular knob. */
.toggle-on { display: flex; flex-direction: row; align-items: center; justify-content: flex-end;
    width: 46px; height: 22px; border-radius: 0; background-color: #2d89ef; border: 1px solid #2d89ef; padding: 2px; }
.toggle-knob { width: 14px; height: 16px; border-radius: 0; background-color: #ffffff; }
.toggle-off { display: flex; flex-direction: row; align-items: center; justify-content: flex-start;
    width: 46px; height: 22px; border-radius: 0; background-color: #ffffff; border: 1px solid #5a5a5a; padding: 2px; }
.toggle-knob-off { width: 14px; height: 16px; border-radius: 0; background-color: #5a5a5a; }

/* ---- Text fields: sharp white boxes, 2px gray border ---- */
.field-col { display: flex; flex-direction: column; }
.field { margin-bottom: 16px; }
.field-filled { background-color: #ffffff; border-radius: 0; border: 2px solid #d0d0d0; padding: 8px 12px; }
.field-outlined { background-color: #ffffff; border: 2px solid #d0d0d0; border-radius: 0; padding: 8px 12px; }
.field-error { background-color: #ffffff; border: 2px solid #da532c; border-radius: 0; padding: 8px 12px; }
.field-label-primary { height: 14px; margin-bottom: 4px; color: #6a6a6a; font-size: 11px; }
.field-label { height: 14px; margin-bottom: 4px; color: #6a6a6a; font-size: 11px; }
.field-label-error { height: 14px; margin-bottom: 4px; color: #da532c; font-size: 11px; }
.field-value { height: 20px; color: #1d1d1d; font-size: 15px; }
.field-placeholder { height: 20px; color: #9a9a9a; font-size: 15px; }
.field-error-text { height: 14px; margin-top: 5px; color: #da532c; font-size: 11px; }

/* ---- Menu / dropdown ---- */
.select-box { display: flex; flex-direction: row; align-items: center; justify-content: space-between;
    width: 260px; height: 44px; background-color: #ffffff; border: 2px solid #d0d0d0; border-radius: 0; padding: 0 12px; }
.select-value { height: 20px; color: #1d1d1d; font-size: 15px; }
.select-chevron { width: 16px; height: 20px; color: #6a6a6a; font-size: 14px; text-align: right; }
.menu-panel { display: flex; flex-direction: column; width: 260px; height: 136px; margin-top: 6px;
    background-color: #ffffff; border: 2px solid #2d89ef; border-radius: 0; }
.menu-item { display: flex; flex-direction: row; align-items: center; justify-content: space-between;
    height: 44px; padding: 0 12px; }
.menu-item.selected { background-color: #2d89ef; }
.menu-text { height: 20px; color: #1d1d1d; font-size: 15px; }
.menu-item.selected .menu-text { color: #ffffff; }
.menu-check { width: 16px; height: 20px; color: #ffffff; font-size: 14px; text-align: right; }
.chips-row { display: flex; flex-direction: row; align-items: center; height: 32px; margin-top: 16px; }
.chip { display: flex; flex-direction: row; align-items: center; justify-content: center; height: 32px;
    border-radius: 0; padding-left: 12px; padding-right: 12px; margin-right: 8px; }
.chip-filled { width: 112px; background-color: #dcdcdc; }
.chip-filled .chip-text { color: #1d1d1d; }
.chip-check { height: 16px; margin-right: 6px; color: #6a6a6a; font-size: 12px; }
.chip-outlined { width: 104px; background-color: #dcdcdc; }
.chip-outlined .chip-text { color: #1d1d1d; }
.chip-tonal { background-color: #dcdcdc; }
.chip-tonal .chip-text { color: #1d1d1d; }
.chip-wide { width: 172px; }
.chip-text { height: 18px; font-size: 14px; font-weight: 600; }

/* ---- Card & list: solid accent live-tile + hard-block avatars ---- */
.mini-card { display: flex; flex-direction: column; height: 208px; background-color: #ffffff; border-radius: 0; }
.mini-cover { display: flex; align-items: center; justify-content: center; height: 96px;
    border-radius: 0; background-color: #2d89ef; }
.mini-cover-text { height: 16px; color: #ffffff; font-size: 13px; font-weight: 600; text-align: center; }
.mini-body { display: flex; flex-direction: column; height: 112px; padding: 14px 2px; }
.mini-title { height: 22px; color: #1d1d1d; font-size: 16px; font-weight: 600; }
.mini-sub { height: 18px; margin-top: 4px; color: #6a6a6a; font-size: 13px; }
.mini-actions { display: flex; flex-direction: row; align-items: center; height: 38px; margin-top: 14px; }
.people { display: flex; flex-direction: column; height: 100px; margin-top: 14px; }
.person { display: flex; flex-direction: row; align-items: center; height: 50px; }
.person-meta { display: flex; flex-direction: column; justify-content: center; height: 50px;
    margin-left: 13px; width: 420px; }
.person-name { height: 18px; color: #1d1d1d; font-size: 14px; }
.person-email { height: 16px; margin-top: 2px; color: #6a6a6a; font-size: 12px; }
.person-menu { width: 24px; height: 20px; color: #9a9a9a; font-size: 18px; text-align: right; }
.avatar { display: flex; align-items: center; justify-content: center;
    width: 34px; height: 34px; border-radius: 0; }
.avatar-text { height: 16px; font-size: 13px; font-weight: 600; text-align: center; color: #ffffff; }
.avatar-ak { background-color: #da532c; }
.avatar-mr { background-color: #00a300; }
.avatar-jp { background-color: #603cba; }
.avatar-more { background-color: #dcdcdc; }
.avatar-more .avatar-text { color: #6a6a6a; }

/* ---- Navigation ---- */
.breadcrumb { display: flex; flex-direction: row; align-items: center; height: 20px; margin-bottom: 18px; }
.crumb { height: 18px; margin-right: 8px; color: #6a6a6a; font-size: 13px; }
.crumb-sep { height: 18px; margin-right: 8px; color: #bdbdbd; font-size: 13px; }
.crumb-active { height: 18px; color: #1d1d1d; font-size: 13px; font-weight: 600; }
.nav-tabs { display: flex; flex-direction: row; height: 42px; margin-bottom: 14px; }
.nav-tab { display: flex; flex-direction: column; align-items: center; justify-content: flex-end;
    width: 96px; }
.nav-tab-label { height: 30px; color: #9a9a9a; font-size: 20px; font-weight: 300; text-align: center; }
.nav-tab-label.active { color: #1d1d1d; }
.nav-tab-underline { width: 0; height: 0; }
.nav-list { display: flex; flex-direction: column; width: 260px; }
.nav-item { display: flex; flex-direction: row; align-items: center; height: 40px;
    border-radius: 0; padding-left: 12px; padding-right: 12px; }
.nav-item.active { background-color: #2d89ef; }
.nav-item-text { height: 20px; margin-left: 10px; color: #5a5a5a; font-size: 14px; }
.nav-item.active .nav-item-text { color: #ffffff; font-weight: 600; }
.dot-on { width: 6px; height: 6px; border-radius: 0; background-color: #ffffff; }
.dot-off { width: 6px; height: 6px; border-radius: 0; background-color: #bdbdbd; }

/* ---- Badges / chips / avatars: solid hard color blocks ---- */
.badge-row { display: flex; flex-direction: row; align-items: center; height: 26px; margin-bottom: 22px; }
.badge { display: flex; align-items: center; justify-content: center; height: 24px;
    border-radius: 0; margin-right: 8px; }
.badge-text { height: 16px; font-size: 12px; font-weight: 600; text-align: center; color: #ffffff; }
.badge-success { width: 76px; background-color: #00a300; }
.badge-warn { width: 84px; background-color: #fa6800; }
.badge-error { width: 66px; background-color: #da532c; }
.badge-neutral { width: 64px; background-color: #dcdcdc; }
.badge-neutral .badge-text { color: #5a5a5a; }
.badge-count { display: flex; align-items: center; justify-content: center;
    width: 24px; height: 20px; border-radius: 0; background-color: #1d1d1d; }
.badge-count-text { height: 14px; color: #ffffff; font-size: 11px; font-weight: 700; text-align: center; }
.avatar-stack { display: flex; flex-direction: row; align-items: center; height: 40px; }
.avatar.stack { border: 2px solid #ffffff; }
.avatar.overlap { margin-left: 6px; }

/* ---- Sliders / progress: thin bar thumb, flat track ---- */
.slider-head { display: flex; flex-direction: row; align-items: center; justify-content: space-between;
    height: 18px; margin-bottom: 10px; }
.slider-name { height: 16px; color: #6a6a6a; font-size: 13px; }
.slider-val { height: 16px; color: #2d89ef; font-size: 13px; font-weight: 600; text-align: right; }
.slider { position: relative; height: 24px; margin-bottom: 24px; }
.slider-track { position: absolute; left: 0; right: 0; top: 10px; height: 4px;
    border-radius: 0; background-color: #cfcfcf; }
.slider-fill { position: absolute; left: 0; top: 10px; width: 55%; height: 4px;
    border-radius: 0; background-color: #2d89ef; }
.slider-knob { position: absolute; left: 54%; top: 1px; width: 8px; height: 22px;
    border-radius: 0; background-color: #2d89ef; }
.progress-label { height: 18px; margin-bottom: 8px; color: #6a6a6a; font-size: 13px; }
.progress { height: 6px; border-radius: 0; background-color: #cfcfcf; margin-bottom: 20px; }
.progress-fill { width: 72%; height: 6px; border-radius: 0; background-color: #2d89ef; }
.spinner-row { display: flex; flex-direction: row; align-items: center; height: 28px; }
.spinner { width: 24px; height: 24px; border-radius: 12px; margin-right: 14px;
    border: 3px solid #cfcfcf; border-top-color: #2d89ef; }
.spinner-text { height: 18px; color: #6a6a6a; font-size: 13px; }

/* ---- Data table: dark header band, ruled rows, no radius ---- */
.table { border-radius: 0; background-color: #ffffff; }
.thead { display: grid; grid-template-columns: 1.6fr 1fr 1fr 0.8fr;
    background-color: #1d1d1d; padding: 12px 16px; border-radius: 0; }
.th { height: 16px; color: #ffffff; font-size: 12px; font-weight: 600; }
.th-right { text-align: right; }
.trow { display: grid; grid-template-columns: 1.6fr 1fr 1fr 0.8fr; align-items: center;
    padding: 12px 16px; border-bottom: 1px solid #e6e6e6; }
.td { height: 20px; color: #1d1d1d; font-size: 14px; }
.td-name { font-weight: 600; }
.td-muted { color: #6a6a6a; }
.td-mono { color: #1d1d1d; }
.td-right { text-align: right; }

/* ---- Feedback ---- */
.feedback-row { display: flex; flex-direction: row; align-items: center; height: 40px; }
";
