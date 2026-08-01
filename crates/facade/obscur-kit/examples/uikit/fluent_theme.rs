#![doc = include_str!("fluent_theme.doc/_module.md")]

#[doc = include_str!("fluent_theme.doc/FLUENT_CSS.md")]
pub(super) const FLUENT_CSS: &str = r#"
* { box-sizing: border-box; }
/* Page = a light "mica" wash: a varied diagonal gradient (blue→neutral→violet→
   neutral→teal) over #eef1f6 so the acrylic frost has something to blur/tint. */
#app { display: flex; flex-direction: column; background-color: #eef1f6;
    background-image: linear-gradient(125deg, #cfe0f7 0%, #eef1f6 27%, #e7d8f2 52%, #eef1f6 74%, #d2ece8 100%); }

/* ---- Switcher bar: acrylic (frosts the mica wash under the title bar) ---- */
.kit-switcher { display: flex; flex-direction: row; align-items: center; height: 60px;
    padding-left: 18px; padding-right: 18px;
    background-color: rgba(255,255,255,0.60); backdrop-filter: blur(24px);
    border-bottom: 1px solid rgba(255,255,255,0.65); }
.kit-brand { display: flex; flex-direction: row; align-items: center; width: 60px; height: 60px; }
.kit-logo { width: 11px; height: 11px; border-radius: 3px; background-color: #0F6CBD; }
.kit-brand-name { width: 40px; height: 16px; margin-left: 8px; color: #242424;
    font-size: 13px; font-weight: 700; }
.kit-tabs { display: flex; flex-direction: row; }
.kit-tab { display: flex; flex-direction: column; align-items: center; justify-content: center;
    width: 108px; height: 60px; }
.kit-tab-label { height: 16px; color: #5a6270; font-size: 12px; font-weight: 600; text-align: center; }
.kit-tab.active .kit-tab-label { color: #0F6CBD; }
.kit-underline { width: 60px; height: 2px; border-radius: 2px; background-color: #0F6CBD; margin-top: 6px; }

/* ---- Gallery frame (transparent, so the mica wash shows behind the cards) ---- */
.m3-page { display: flex; flex-direction: column; padding: 34px 40px 60px; }
.m3-head { display: flex; flex-direction: column; height: 108px; margin-bottom: 26px; }
.m3-eyebrow { height: 16px; color: #5a6270; font-size: 11px; font-weight: 600; }
.m3-title { height: 48px; color: #242424; font-size: 38px; font-weight: 600; }
.m3-sub { height: 22px; color: #4b5563; font-size: 15px; }

.gallery-grid { display: flex; flex-direction: column; }
.row { display: flex; flex-direction: row; align-items: stretch; margin-bottom: 24px; }
.row.r1 { height: 232px; }
.row.r2 { height: 336px; }
.row.r3 { height: 420px; }
.row.r4 { height: 252px; }

/* Cards = ACRYLIC: translucent white + backdrop blur + hairline light border +
   soft depth shadow. The mica gradient frosts + tints through each panel. */
.card { background-color: rgba(255,255,255,0.55); backdrop-filter: blur(24px);
    border: 1px solid rgba(255,255,255,0.65); border-radius: 12px; padding: 22px;
    box-shadow: 0 8px 24px rgba(15,30,60,0.10), 0 1px 2px rgba(0,0,0,0.06); }
.card.col { width: 588px; margin-right: 24px; }
.card.full { width: 1200px; margin-bottom: 24px; }
.card.full.table-c { height: 288px; }
.card.full.feedback-c { height: 132px; }
.card-label { height: 16px; margin-bottom: 16px; color: #6b7280; font-size: 11px; font-weight: 600; }

/* ---- Buttons: accent-filled primary, translucent standard, subtle text ---- */
.btn-row { display: flex; flex-direction: row; align-items: center; height: 56px; }
.btn { display: flex; flex-direction: row; align-items: center; justify-content: center;
    height: 40px; border-radius: 6px; padding-left: 20px; padding-right: 20px; margin-right: 10px; }
.btn-label { height: 18px; font-size: 14px; font-weight: 600; text-align: center; }
.btn-filled { width: 110px; background-color: #0F6CBD; box-shadow: 0 2px 4px rgba(15,108,189,0.32); }
.btn-filled .btn-label { color: #ffffff; }
.btn-tonal { width: 104px; background-color: rgba(255,255,255,0.70); border: 1px solid rgba(0,0,0,0.14); }
.btn-tonal .btn-label { color: #242424; }
.btn-outlined { width: 128px; background-color: rgba(255,255,255,0.55); border: 1px solid #0F6CBD; }
.btn-outlined .btn-label { color: #0F6CBD; }
.btn-text { width: 92px; padding-left: 16px; padding-right: 16px; background-color: rgba(255,255,255,0.0); }
.btn-text .btn-label { color: #0F6CBD; }
.btn-dialog { width: 152px; }
.btn-elevated { width: 132px; background-color: rgba(255,255,255,0.70); border: 1px solid rgba(0,0,0,0.14);
    box-shadow: 0 4px 12px rgba(15,30,60,0.10); }
.btn-elevated .btn-label { color: #0F6CBD; }
.btn-disabled { width: 128px; background-color: rgba(0,0,0,0.03); border: 1px solid rgba(0,0,0,0.08); }
.btn-disabled .btn-label { color: #bdbdbd; }
/* FAB -> the Fluent accent icon-square add affordance. */
.fab { display: flex; flex-direction: row; align-items: center; justify-content: center;
    width: 56px; height: 56px; border-radius: 8px; background-color: #0F6CBD;
    box-shadow: 0 4px 12px rgba(15,108,189,0.34); }
.fab-glyph { height: 30px; color: #ffffff; font-size: 26px; font-weight: 400; text-align: center; }
.btn-sm { height: 38px; }
.btn-wide { width: 168px; }

/* ---- Selection ---- */
.sel-cols { display: flex; flex-direction: row; }
.sel-col { display: flex; flex-direction: column; width: 168px; }
.sel-item { display: flex; flex-direction: row; align-items: center; height: 34px; }
.sel-col .sel-item { margin-bottom: 14px; }
.sel-col.wide-gap .sel-item { margin-bottom: 18px; height: 32px; }
.sel-text { height: 20px; margin-left: 11px; color: #242424; font-size: 14px; }
.checkbox-on { display: flex; align-items: center; justify-content: center;
    width: 19px; height: 19px; border-radius: 4px; background-color: #0F6CBD; }
.check-glyph { height: 14px; color: #ffffff; font-size: 12px; text-align: center; }
.checkbox-off { width: 19px; height: 19px; border-radius: 4px; border: 2px solid #616161; background-color: rgba(255,255,255,0.60); }
.checkbox-disabled { width: 19px; height: 19px; border-radius: 4px; border: 2px solid #d4d4d4; background-color: rgba(0,0,0,0.03); }
.radio-on { display: flex; align-items: center; justify-content: center;
    width: 20px; height: 20px; border-radius: 10px; border: 2px solid #0F6CBD; background-color: rgba(255,255,255,0.60); }
.radio-dot { width: 10px; height: 10px; border-radius: 5px; background-color: #0F6CBD; }
.radio-off { width: 20px; height: 20px; border-radius: 10px; border: 2px solid #616161; background-color: rgba(255,255,255,0.60); }
.toggle-on { display: flex; flex-direction: row; align-items: center; justify-content: flex-end;
    width: 44px; height: 22px; border-radius: 11px; background-color: #0F6CBD; padding: 4px; }
.toggle-knob { width: 14px; height: 14px; border-radius: 7px; background-color: #ffffff; }
.toggle-off { display: flex; flex-direction: row; align-items: center; justify-content: flex-start;
    width: 44px; height: 22px; border-radius: 11px; background-color: rgba(255,255,255,0.50);
    border: 1px solid #616161; padding: 4px; }
.toggle-knob-off { width: 13px; height: 13px; border-radius: 7px; background-color: #616161; }

/* ---- Text fields: translucent inputs, accent underline on the primary/error ---- */
.field-col { display: flex; flex-direction: column; }
.field { margin-bottom: 16px; }
.field-filled { background-color: rgba(255,255,255,0.70); border: 1px solid rgba(0,0,0,0.14);
    border-bottom: 2px solid #0F6CBD; border-radius: 5px; padding: 9px 12px; }
.field-outlined { background-color: rgba(255,255,255,0.70); border: 1px solid rgba(0,0,0,0.14); border-radius: 5px; padding: 9px 12px; }
.field-error { background-color: rgba(255,255,255,0.70); border: 1px solid #C50F1F; border-bottom: 2px solid #C50F1F; border-radius: 5px; padding: 9px 12px; }
.field-label-primary { height: 14px; margin-bottom: 4px; color: #4b5563; font-size: 11px; font-weight: 600; }
.field-label { height: 14px; margin-bottom: 4px; color: #4b5563; font-size: 11px; font-weight: 600; }
.field-label-error { height: 14px; margin-bottom: 4px; color: #C50F1F; font-size: 11px; font-weight: 600; }
.field-value { height: 20px; color: #242424; font-size: 15px; }
.field-placeholder { height: 20px; color: #6b7280; font-size: 15px; }
.field-error-text { height: 14px; margin-top: 5px; color: #C50F1F; font-size: 11px; }

/* ---- Menu / dropdown: acrylic panel over the mica wash ---- */
.select-box { display: flex; flex-direction: row; align-items: center; justify-content: space-between;
    width: 260px; height: 44px; background-color: rgba(255,255,255,0.70); border: 1px solid rgba(0,0,0,0.14);
    border-bottom: 2px solid #0F6CBD; border-radius: 5px; padding: 0 14px; }
.select-value { height: 20px; color: #242424; font-size: 15px; }
.select-chevron { width: 16px; height: 20px; color: #6b7280; font-size: 14px; text-align: right; }
.menu-panel { display: flex; flex-direction: column; width: 260px; height: 136px; margin-top: 8px;
    background-color: rgba(250,251,253,0.82); backdrop-filter: blur(24px);
    border: 1px solid rgba(255,255,255,0.70); border-radius: 8px; padding-top: 6px; padding-bottom: 6px;
    box-shadow: 0 12px 32px rgba(15,30,60,0.20); }
.menu-item { display: flex; flex-direction: row; align-items: center; justify-content: space-between;
    height: 40px; padding: 0 12px; }
.menu-item.selected { background-color: rgba(15,108,189,0.10); border-radius: 6px; }
.menu-text { height: 20px; color: #242424; font-size: 15px; }
.menu-check { width: 16px; height: 20px; color: #0F6CBD; font-size: 14px; text-align: right; }
.chips-row { display: flex; flex-direction: row; align-items: center; height: 32px; margin-top: 16px; }
.chip { display: flex; flex-direction: row; align-items: center; justify-content: center; height: 30px;
    border-radius: 15px; padding-left: 14px; padding-right: 14px; margin-right: 8px; }
.chip-filled { width: 112px; background-color: rgba(15,108,189,0.14); }
.chip-filled .chip-text { color: #0F6CBD; }
.chip-check { height: 16px; margin-right: 6px; color: #0F6CBD; font-size: 12px; }
.chip-outlined { width: 104px; background-color: rgba(255,255,255,0.70); border: 1px solid rgba(0,0,0,0.14); }
.chip-outlined .chip-text { color: #4b5563; }
.chip-tonal { background-color: rgba(15,108,189,0.14); }
.chip-tonal .chip-text { color: #0F6CBD; }
.chip-wide { width: 172px; }
.chip-text { height: 18px; font-size: 13px; font-weight: 600; }

/* ---- Card & list: a nested translucent tile inside the acrylic card ---- */
.mini-card { display: flex; flex-direction: column; height: 208px; background-color: rgba(255,255,255,0.50);
    border: 1px solid rgba(255,255,255,0.70); border-radius: 10px; box-shadow: 0 4px 12px rgba(15,30,60,0.08); }
.mini-cover { display: flex; align-items: center; justify-content: center; height: 84px;
    border-radius: 10px 10px 0 0; background-image: linear-gradient(120deg, #0F6CBD, #9240C5); }
.mini-cover-text { height: 16px; color: #ffffff; font-size: 11px; text-align: center; }
.mini-body { display: flex; flex-direction: column; height: 124px; padding: 16px; }
.mini-title { height: 22px; color: #242424; font-size: 16px; font-weight: 600; }
.mini-sub { height: 18px; margin-top: 4px; color: #4b5563; font-size: 13px; }
.mini-actions { display: flex; flex-direction: row; align-items: center; height: 38px; margin-top: 14px; }
.people { display: flex; flex-direction: column; height: 100px; margin-top: 14px; }
.person { display: flex; flex-direction: row; align-items: center; height: 50px; }
.person-meta { display: flex; flex-direction: column; justify-content: center; height: 50px;
    margin-left: 13px; width: 420px; }
.person-name { height: 18px; color: #242424; font-size: 14px; }
.person-email { height: 16px; margin-top: 2px; color: #4b5563; font-size: 12px; }
.person-menu { width: 24px; height: 20px; color: #9ca3af; font-size: 18px; text-align: right; }
.avatar { display: flex; align-items: center; justify-content: center;
    width: 36px; height: 36px; border-radius: 18px; }
.avatar-text { height: 16px; font-size: 13px; font-weight: 600; text-align: center; color: #ffffff; }
.avatar-ak { background-color: #0F6CBD; }
.avatar-mr { background-color: #9240C5; }
.avatar-jp { background-color: #107C10; }
.avatar-more { background-color: rgba(15,108,189,0.16); }
.avatar-more .avatar-text { color: #0F6CBD; }

/* ---- Navigation ---- */
.breadcrumb { display: flex; flex-direction: row; align-items: center; height: 20px; margin-bottom: 18px; }
.crumb { height: 18px; margin-right: 8px; color: #4b5563; font-size: 13px; }
.crumb-sep { height: 18px; margin-right: 8px; color: #c4c9d1; font-size: 13px; }
.crumb-active { height: 18px; color: #0F6CBD; font-size: 13px; font-weight: 600; }
.nav-tabs { display: flex; flex-direction: row; height: 42px;
    border-bottom: 1px solid rgba(0,0,0,0.08); margin-bottom: 14px; }
.nav-tab { display: flex; flex-direction: column; align-items: center; justify-content: flex-end;
    width: 96px; }
.nav-tab-label { height: 24px; color: #4b5563; font-size: 14px; font-weight: 600; text-align: center; }
.nav-tab-label.active { color: #0F6CBD; }
.nav-tab-underline { width: 68px; height: 2px; border-radius: 3px; background-color: #0F6CBD; }
.nav-list { display: flex; flex-direction: column; width: 260px; }
.nav-item { display: flex; flex-direction: row; align-items: center; height: 40px;
    border-radius: 6px; padding-left: 14px; padding-right: 14px; margin-bottom: 3px; }
.nav-item.active { background-color: rgba(15,108,189,0.12); }
.nav-item-text { height: 20px; margin-left: 12px; color: #4b5563; font-size: 14px; }
.nav-item.active .nav-item-text { color: #0F6CBD; font-weight: 600; }
.dot-on { width: 8px; height: 8px; border-radius: 2px; background-color: #0F6CBD; }
.dot-off { width: 8px; height: 8px; border-radius: 2px; background-color: #c4c9d1; }

/* ---- Badges / chips / avatars: soft tonal pill tags ---- */
.badge-row { display: flex; flex-direction: row; align-items: center; height: 26px; margin-bottom: 22px; }
.badge { display: flex; align-items: center; justify-content: center; height: 24px;
    border-radius: 12px; margin-right: 8px; }
.badge-text { height: 16px; font-size: 12px; font-weight: 600; text-align: center; }
.badge-success { width: 76px; background-color: rgba(16,124,16,0.14); }
.badge-success .badge-text { color: #0E700E; }
.badge-warn { width: 84px; background-color: rgba(157,93,0,0.15); }
.badge-warn .badge-text { color: #8A5300; }
.badge-error { width: 66px; background-color: rgba(197,15,31,0.13); }
.badge-error .badge-text { color: #C50F1F; }
.badge-neutral { width: 64px; background-color: rgba(0,0,0,0.06); }
.badge-neutral .badge-text { color: #4b5563; }
.badge-count { display: flex; align-items: center; justify-content: center;
    width: 24px; height: 20px; border-radius: 10px; background-color: #C50F1F; }
.badge-count-text { height: 14px; color: #ffffff; font-size: 11px; font-weight: 700; text-align: center; }
.avatar-stack { display: flex; flex-direction: row; align-items: center; height: 40px; }
.avatar.stack { border: 2px solid rgba(255,255,255,0.80); }
.avatar.overlap { margin-left: -10px; }

/* ---- Sliders / progress ---- */
.slider-head { display: flex; flex-direction: row; align-items: center; justify-content: space-between;
    height: 18px; margin-bottom: 10px; }
.slider-name { height: 16px; color: #4b5563; font-size: 13px; }
.slider-val { height: 16px; color: #0F6CBD; font-size: 13px; font-weight: 600; text-align: right; }
.slider { position: relative; height: 24px; margin-bottom: 24px; }
.slider-track { position: absolute; left: 0; right: 0; top: 10px; height: 4px;
    border-radius: 2px; background-color: rgba(0,0,0,0.12); }
.slider-fill { position: absolute; left: 0; top: 10px; width: 55%; height: 4px;
    border-radius: 2px; background-color: #0F6CBD; }
.slider-knob { position: absolute; left: 53%; top: 2px; width: 20px; height: 20px;
    border-radius: 10px; background-color: #ffffff; border: 5px solid #0F6CBD; box-shadow: 0 1px 3px rgba(0,0,0,0.20); }
.progress-label { height: 18px; margin-bottom: 8px; color: #4b5563; font-size: 13px; }
.progress { height: 4px; border-radius: 2px; background-color: rgba(0,0,0,0.12); margin-bottom: 20px; }
.progress-fill { width: 72%; height: 4px; border-radius: 2px; background-color: #0F6CBD; }
.spinner-row { display: flex; flex-direction: row; align-items: center; height: 28px; }
.spinner { width: 28px; height: 28px; border-radius: 14px; margin-right: 14px;
    border: 3px solid rgba(0,0,0,0.10); border-top-color: #0F6CBD; }
.spinner-text { height: 18px; color: #4b5563; font-size: 13px; }

/* ---- Data table ---- */
.table { border: 1px solid rgba(0,0,0,0.08); border-radius: 10px; }
.thead { display: grid; grid-template-columns: 1.6fr 1fr 1fr 0.8fr;
    background-color: rgba(255,255,255,0.50); padding: 13px 18px; border-radius: 10px 10px 0 0; }
.th { height: 16px; color: #4b5563; font-size: 12px; font-weight: 600; }
.th-right { text-align: right; }
.trow { display: grid; grid-template-columns: 1.6fr 1fr 1fr 0.8fr; align-items: center;
    padding: 13px 18px; border-top: 1px solid rgba(0,0,0,0.06); }
.td { height: 20px; color: #242424; font-size: 14px; }
.td-name { color: #242424; font-weight: 600; }
.td-muted { color: #4b5563; }
.td-mono { color: #242424; }
.td-right { text-align: right; }

/* ---- Feedback ---- */
.feedback-row { display: flex; flex-direction: row; align-items: center; height: 40px; }
"#;
