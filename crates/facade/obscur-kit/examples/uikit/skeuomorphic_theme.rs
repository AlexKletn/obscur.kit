#![doc = include_str!("skeuomorphic_theme.doc/_module.md")]

#[doc = include_str!("skeuomorphic_theme.doc/SKEUOMORPHIC_CSS.md")]
pub(super) const SKEUOMORPHIC_CSS: &str = r"
* { box-sizing: border-box; }
/* Page: the brushed-beige backdrop gradient (real ramp on Vello, first stop on CPU). */
#app { display: flex; flex-direction: column; background-image: linear-gradient(#cfcabb, #bdb8a8); }

/* ---- Switcher bar: a dark brushed-metal strip with a top gloss line ---- */
.kit-switcher { display: flex; flex-direction: row; align-items: center; height: 60px;
    padding-left: 18px; padding-right: 18px;
    background-image: linear-gradient(#5a5a52, #33332e);
    border-bottom: 1px solid #22221e;
    box-shadow: inset 0 1px 0 rgba(255,255,255,0.18), 0 2px 4px rgba(0,0,0,0.35); }
.kit-brand { display: flex; flex-direction: row; align-items: center; width: 60px; height: 60px; }
.kit-logo { width: 11px; height: 11px; border-radius: 3px;
    background-image: linear-gradient(#f5cd6b, #e0a82f); border: 1px solid #b8860b;
    box-shadow: inset 0 1px 0 rgba(255,255,255,0.6); }
.kit-brand-name { width: 40px; height: 16px; margin-left: 8px; color: #f2efe8;
    font-size: 13px; font-weight: 700; }
.kit-tabs { display: flex; flex-direction: row; }
.kit-tab { display: flex; flex-direction: column; align-items: center; justify-content: center;
    width: 108px; height: 60px; }
.kit-tab-label { height: 16px; color: #b8b3a4; font-size: 12px; font-weight: 500; text-align: center; }
.kit-tab.active .kit-tab-label { color: #ffffff; }
.kit-underline { width: 60px; height: 3px; border-radius: 2px; margin-top: 6px;
    background-image: linear-gradient(#7eb4f2, #3a78db);
    box-shadow: 0 1px 2px rgba(0,0,0,0.4); }

/* ---- Gallery frame ---- */
.m3-page { display: flex; flex-direction: column; padding: 34px 40px 60px; }
.m3-head { display: flex; flex-direction: column; height: 108px; margin-bottom: 26px; }
.m3-eyebrow { height: 16px; color: #8a8474; font-size: 11px; font-weight: 700; }
.m3-title { height: 48px; color: #3a362e; font-size: 38px; font-weight: 700; }
.m3-sub { height: 22px; color: #6b6658; font-size: 15px; }

.gallery-grid { display: flex; flex-direction: column; }
.row { display: flex; flex-direction: row; align-items: stretch; margin-bottom: 24px; }
.row.r1 { height: 232px; }
.row.r2 { height: 336px; }
.row.r3 { height: 420px; }
.row.r4 { height: 252px; }

/* Cards: the molded beige panel — top-light face, bevel border, top gloss line + cast shadow. */
.card { padding: 22px; border-radius: 12px; border: 1px solid #a9a496;
    background-image: linear-gradient(#f2efe8, #e0dccf);
    box-shadow: inset 0 1px 0 rgba(255,255,255,0.8), 0 2px 5px rgba(0,0,0,0.25); }
.card.col { width: 588px; margin-right: 24px; }
.card.full { width: 1200px; margin-bottom: 24px; }
.card.full.table-c { height: 288px; }
.card.full.feedback-c { height: 132px; }
.card-label { height: 16px; margin-bottom: 16px; color: #8a8474; font-size: 11px; font-weight: 700; }

/* ---- Buttons: glossy hardware keys ---- */
.btn-row { display: flex; flex-direction: row; align-items: center; height: 56px; }
.btn { display: flex; flex-direction: row; align-items: center; justify-content: center;
    height: 40px; border-radius: 10px; padding-left: 22px; padding-right: 22px; margin-right: 10px;
    border: 1px solid #a9a496;
    background-image: linear-gradient(#fbfbfa, #e6e3da);
    box-shadow: inset 0 1px 0 rgba(255,255,255,0.9), 0 1px 2px rgba(0,0,0,0.25); }
.btn-label { height: 18px; font-size: 14px; font-weight: 700; text-align: center; color: #4a4636; }
/* Primary blue: glossy candy button with a darker bevel + white gloss line. */
.btn-filled { width: 110px; border: 1px solid #2560bf;
    background-image: linear-gradient(#6ba8ef, #2f6fd6);
    box-shadow: inset 0 1px 0 rgba(255,255,255,0.45), 0 1px 3px rgba(0,0,0,0.35); }
.btn-filled .btn-label { color: #ffffff; }
.btn-tonal { width: 104px; border: 1px solid #9bb7dd;
    background-image: linear-gradient(#dbe7f8, #b9cdec); }
.btn-tonal .btn-label { color: #234a86; }
.btn-outlined { width: 128px; border: 1px solid #a9a496;
    background-image: linear-gradient(#f2efe8, #e0dccf); }
.btn-outlined .btn-label { color: #4a4636; }
.btn-text { width: 92px; padding-left: 16px; padding-right: 16px;
    border: 1px solid transparent; background-image: linear-gradient(#efece4, #efece4);
    box-shadow: none; }
.btn-text .btn-label { color: #2f6fd6; }
.btn-dialog { width: 152px; }
/* Elevated: same glossy face, a heavier cast shadow to float it. */
.btn-elevated { width: 132px; border: 1px solid #a9a496;
    background-image: linear-gradient(#fbfbfa, #e6e3da);
    box-shadow: inset 0 1px 0 rgba(255,255,255,0.9), 0 3px 6px rgba(0,0,0,0.3); }
.btn-elevated .btn-label { color: #4a4636; }
/* Disabled: flat, no gloss, no shadow — reads as inert. */
.btn-disabled { width: 128px; border: 1px solid #c7c3b6;
    background-image: linear-gradient(#e4e1d8, #e4e1d8); box-shadow: none; }
.btn-disabled .btn-label { color: #a7a394; }
/* FAB: a glossy blue puck. */
.fab { display: flex; flex-direction: row; align-items: center; justify-content: center;
    width: 56px; height: 56px; border-radius: 16px; border: 1px solid #2560bf;
    background-image: linear-gradient(#6ba8ef, #2f6fd6);
    box-shadow: inset 0 1px 0 rgba(255,255,255,0.45), 0 3px 6px rgba(0,0,0,0.35); }
.fab-glyph { height: 30px; color: #ffffff; font-size: 26px; font-weight: 400; text-align: center; }
.btn-sm { height: 38px; }
.btn-wide { width: 168px; }

/* ---- Selection: glossy checks / radios / a recessed toggle groove ---- */
.sel-cols { display: flex; flex-direction: row; }
.sel-col { display: flex; flex-direction: column; width: 168px; }
.sel-item { display: flex; flex-direction: row; align-items: center; height: 34px; }
.sel-col .sel-item { margin-bottom: 14px; }
.sel-col.wide-gap .sel-item { margin-bottom: 18px; height: 32px; }
.sel-text { height: 20px; margin-left: 11px; color: #4a4636; font-size: 14px; }
.checkbox-on { display: flex; align-items: center; justify-content: center;
    width: 18px; height: 18px; border-radius: 4px; border: 1px solid #2560bf;
    background-image: linear-gradient(#6ba8ef, #2f6fd6);
    box-shadow: inset 0 1px 0 rgba(255,255,255,0.4), 0 1px 1px rgba(0,0,0,0.3); }
.check-glyph { height: 14px; color: #ffffff; font-size: 12px; text-align: center; }
.checkbox-off { width: 18px; height: 18px; border-radius: 4px; border: 1px solid #a9a496;
    background-image: linear-gradient(#e3e0d8, #fcfbf8);
    box-shadow: inset 0 1px 2px rgba(0,0,0,0.25); }
.checkbox-disabled { width: 18px; height: 18px; border-radius: 4px; border: 1px solid #c7c3b6;
    background-image: linear-gradient(#e4e1d8, #e4e1d8); }
.radio-on { display: flex; align-items: center; justify-content: center;
    width: 20px; height: 20px; border-radius: 10px; border: 1px solid #a9a496;
    background-image: linear-gradient(#e3e0d8, #fcfbf8);
    box-shadow: inset 0 1px 2px rgba(0,0,0,0.2); }
.radio-dot { width: 10px; height: 10px; border-radius: 5px; border: 1px solid #2560bf;
    background-image: linear-gradient(#6ba8ef, #2f6fd6);
    box-shadow: inset 0 1px 0 rgba(255,255,255,0.5); }
.radio-off { width: 20px; height: 20px; border-radius: 10px; border: 1px solid #a9a496;
    background-image: linear-gradient(#e3e0d8, #fcfbf8);
    box-shadow: inset 0 1px 2px rgba(0,0,0,0.2); }
/* Toggle: a sunken groove (inset well) with a raised glossy knob. */
.toggle-on { display: flex; flex-direction: row; align-items: center; justify-content: flex-end;
    width: 52px; height: 32px; border-radius: 16px; padding: 4px; border: 1px solid #2560bf;
    background-image: linear-gradient(#3a78db, #6ba8ef);
    box-shadow: inset 0 2px 3px rgba(0,0,0,0.35); }
.toggle-knob { width: 24px; height: 24px; border-radius: 12px; border: 1px solid #b8b3a4;
    background-image: linear-gradient(#ffffff, #dedacf);
    box-shadow: inset 0 1px 0 rgba(255,255,255,0.9), 0 1px 2px rgba(0,0,0,0.4); }
.toggle-off { display: flex; flex-direction: row; align-items: center; justify-content: flex-start;
    width: 52px; height: 32px; border-radius: 16px; padding: 6px; border: 1px solid #a9a496;
    background-image: linear-gradient(#c9c5bb, #ddd9d0);
    box-shadow: inset 0 2px 3px rgba(0,0,0,0.3); }
.toggle-knob-off { width: 16px; height: 16px; border-radius: 8px; border: 1px solid #b8b3a4;
    background-image: linear-gradient(#ffffff, #dedacf);
    box-shadow: 0 1px 2px rgba(0,0,0,0.35); }

/* ---- Text fields: recessed wells (inset shadow, light face, bevel border) ---- */
.field-col { display: flex; flex-direction: column; }
.field { margin-bottom: 16px; }
.field-filled { border-radius: 8px; padding: 8px 14px 6px; border: 1px solid #a9a496;
    background-image: linear-gradient(#e3e0d8, #fcfbf8);
    box-shadow: inset 0 2px 3px rgba(0,0,0,0.22); }
.field-outlined { border-radius: 8px; padding: 10px 14px; border: 1px solid #a9a496;
    background-image: linear-gradient(#e3e0d8, #fcfbf8);
    box-shadow: inset 0 2px 3px rgba(0,0,0,0.18); }
.field-error { border-radius: 8px; padding: 10px 14px; border: 1px solid #c0392b;
    background-image: linear-gradient(#f6e2de, #fcf7f6);
    box-shadow: inset 0 2px 3px rgba(0,0,0,0.18); }
.field-label-primary { height: 14px; margin-bottom: 4px; color: #2f6fd6; font-size: 11px; font-weight: 700; }
.field-label { height: 14px; margin-bottom: 4px; color: #6b6658; font-size: 11px; font-weight: 700; }
.field-label-error { height: 14px; margin-bottom: 4px; color: #c0392b; font-size: 11px; font-weight: 700; }
.field-value { height: 20px; color: #3a362e; font-size: 15px; }
.field-placeholder { height: 20px; color: #8a8474; font-size: 15px; }
.field-error-text { height: 14px; margin-top: 5px; color: #c0392b; font-size: 11px; }

/* ---- Menu / dropdown ---- */
.select-box { display: flex; flex-direction: row; align-items: center; justify-content: space-between;
    width: 260px; height: 44px; border-radius: 8px; padding: 0 14px; border: 1px solid #a9a496;
    background-image: linear-gradient(#fbfbfa, #e6e3da);
    box-shadow: inset 0 1px 0 rgba(255,255,255,0.9), 0 1px 2px rgba(0,0,0,0.22); }
.select-value { height: 20px; color: #3a362e; font-size: 15px; }
.select-chevron { width: 16px; height: 20px; color: #6b6658; font-size: 14px; text-align: right; }
.menu-panel { display: flex; flex-direction: column; width: 260px; height: 136px; margin-top: 8px;
    border-radius: 10px; padding-top: 8px; padding-bottom: 8px; border: 1px solid #a9a496;
    background-image: linear-gradient(#f6f4ee, #e4e0d5);
    box-shadow: inset 0 1px 0 rgba(255,255,255,0.8), 0 6px 14px rgba(0,0,0,0.3); }
.menu-item { display: flex; flex-direction: row; align-items: center; justify-content: space-between;
    height: 40px; padding: 0 16px; }
.menu-item.selected { border-radius: 6px;
    background-image: linear-gradient(#6ba8ef, #2f6fd6);
    box-shadow: inset 0 1px 0 rgba(255,255,255,0.35); }
.menu-text { height: 20px; color: #3a362e; font-size: 15px; }
.menu-item.selected .menu-text { color: #ffffff; }
.menu-check { width: 16px; height: 20px; color: #ffffff; font-size: 14px; text-align: right; }
.chips-row { display: flex; flex-direction: row; align-items: center; height: 32px; margin-top: 16px; }
.chip { display: flex; flex-direction: row; align-items: center; justify-content: center; height: 32px;
    border-radius: 16px; padding-left: 14px; padding-right: 14px; margin-right: 8px;
    border: 1px solid #a9a496;
    background-image: linear-gradient(#fbfbfa, #e6e3da);
    box-shadow: inset 0 1px 0 rgba(255,255,255,0.9), 0 1px 2px rgba(0,0,0,0.2); }
.chip-filled { width: 112px; border: 1px solid #9bb7dd;
    background-image: linear-gradient(#dbe7f8, #b9cdec); }
.chip-filled .chip-text { color: #234a86; }
.chip-check { height: 16px; margin-right: 6px; color: #234a86; font-size: 12px; }
.chip-outlined { width: 104px; }
.chip-outlined .chip-text { color: #4a4636; }
.chip-tonal { background-image: linear-gradient(#dbe7f8, #b9cdec); border: 1px solid #9bb7dd; }
.chip-tonal .chip-text { color: #234a86; }
.chip-wide { width: 172px; }
.chip-text { height: 18px; font-size: 14px; font-weight: 700; }

/* ---- Card & list ---- */
.mini-card { display: flex; flex-direction: column; height: 208px; border-radius: 10px;
    border: 1px solid #a9a496; background-image: linear-gradient(#f6f4ee, #e8e6df);
    box-shadow: inset 0 1px 0 rgba(255,255,255,0.7), 0 2px 4px rgba(0,0,0,0.22); }
.mini-cover { display: flex; align-items: center; justify-content: center; height: 84px;
    border-radius: 10px 10px 0 0;
    background-image: linear-gradient(#7eb4f2, #3a78db);
    box-shadow: inset 0 1px 0 rgba(255,255,255,0.4); }
.mini-cover-text { height: 16px; color: #ffffff; font-size: 11px; text-align: center; }
.mini-body { display: flex; flex-direction: column; height: 124px; padding: 16px; }
.mini-title { height: 22px; color: #3a362e; font-size: 16px; font-weight: 700; }
.mini-sub { height: 18px; margin-top: 4px; color: #6b6658; font-size: 13px; }
.mini-actions { display: flex; flex-direction: row; align-items: center; height: 38px; margin-top: 14px; }
.people { display: flex; flex-direction: column; height: 100px; margin-top: 14px; }
.person { display: flex; flex-direction: row; align-items: center; height: 50px; }
.person-meta { display: flex; flex-direction: column; justify-content: center; height: 50px;
    margin-left: 13px; width: 420px; }
.person-name { height: 18px; color: #3a362e; font-size: 14px; font-weight: 700; }
.person-email { height: 16px; margin-top: 2px; color: #6b6658; font-size: 12px; }
.person-menu { width: 24px; height: 20px; color: #8a8474; font-size: 18px; text-align: right; }
.avatar { display: flex; align-items: center; justify-content: center;
    width: 36px; height: 36px; border-radius: 18px; border: 1px solid rgba(0,0,0,0.2);
    box-shadow: inset 0 1px 0 rgba(255,255,255,0.4), 0 1px 2px rgba(0,0,0,0.3); }
.avatar-text { height: 16px; font-size: 13px; font-weight: 700; text-align: center; color: #ffffff; }
.avatar-ak { background-image: linear-gradient(#6ba8ef, #2f6fd6); }
.avatar-mr { background-image: linear-gradient(#c98a97, #a04a5a); }
.avatar-jp { background-image: linear-gradient(#7ec46a, #4e9a3a); }
.avatar-more { background-image: linear-gradient(#f5cd6b, #e0a82f); }
.avatar-more .avatar-text { color: #6b4a00; }

/* ---- Navigation ---- */
.breadcrumb { display: flex; flex-direction: row; align-items: center; height: 20px; margin-bottom: 18px; }
.crumb { height: 18px; margin-right: 8px; color: #6b6658; font-size: 13px; }
.crumb-sep { height: 18px; margin-right: 8px; color: #a9a496; font-size: 13px; }
.crumb-active { height: 18px; color: #2f6fd6; font-size: 13px; font-weight: 700; }
.nav-tabs { display: flex; flex-direction: row; height: 42px;
    border-bottom: 1px solid #c7c3b6; margin-bottom: 14px; }
.nav-tab { display: flex; flex-direction: column; align-items: center; justify-content: flex-end;
    width: 96px; }
.nav-tab-label { height: 24px; color: #6b6658; font-size: 14px; font-weight: 700; text-align: center; }
.nav-tab-label.active { color: #2f6fd6; }
.nav-tab-underline { width: 72px; height: 3px; border-radius: 3px 3px 0 0;
    background-image: linear-gradient(#7eb4f2, #3a78db); }
.nav-list { display: flex; flex-direction: column; width: 260px; }
.nav-item { display: flex; flex-direction: row; align-items: center; height: 44px;
    border-radius: 10px; padding-left: 16px; padding-right: 16px; margin-bottom: 4px;
    border: 1px solid transparent; }
.nav-item.active { border: 1px solid #a9a496;
    background-image: linear-gradient(#f2efe8, #e0dccf);
    box-shadow: inset 0 1px 0 rgba(255,255,255,0.8), 0 1px 2px rgba(0,0,0,0.18); }
.nav-item-text { height: 20px; margin-left: 12px; color: #6b6658; font-size: 14px; }
.nav-item.active .nav-item-text { color: #3a362e; font-weight: 700; }
.dot-on { width: 9px; height: 9px; border-radius: 5px; border: 1px solid #2560bf;
    background-image: linear-gradient(#6ba8ef, #2f6fd6); }
.dot-off { width: 9px; height: 9px; border-radius: 5px; border: 1px solid #a9a496;
    background-image: linear-gradient(#e3e0d8, #c9c5bb); }

/* ---- Badges / chips / avatars: glossy candy pills ---- */
.badge-row { display: flex; flex-direction: row; align-items: center; height: 26px; margin-bottom: 22px; }
.badge { display: flex; align-items: center; justify-content: center; height: 26px;
    border-radius: 13px; margin-right: 8px; border: 1px solid rgba(0,0,0,0.25);
    box-shadow: inset 0 1px 0 rgba(255,255,255,0.45), 0 1px 2px rgba(0,0,0,0.25); }
.badge-text { height: 16px; font-size: 12px; font-weight: 700; text-align: center; color: #ffffff; }
.badge-success { width: 76px; background-image: linear-gradient(#7ec46a, #4e9a3a); }
.badge-warn { width: 84px; background-image: linear-gradient(#f5cd6b, #e0a82f); }
.badge-warn .badge-text { color: #5a3d00; }
.badge-error { width: 66px; background-image: linear-gradient(#e8896f, #c0392b); }
.badge-neutral { width: 64px; background-image: linear-gradient(#e3e0d8, #c9c5bb); }
.badge-neutral .badge-text { color: #4a4636; }
.badge-count { display: flex; align-items: center; justify-content: center;
    width: 24px; height: 20px; border-radius: 10px; border: 1px solid #8f1f14;
    background-image: linear-gradient(#e8896f, #c0392b);
    box-shadow: inset 0 1px 0 rgba(255,255,255,0.4); }
.badge-count-text { height: 14px; color: #ffffff; font-size: 11px; font-weight: 700; text-align: center; }
.avatar-stack { display: flex; flex-direction: row; align-items: center; height: 40px; }
.avatar.stack { border: 2px solid #f2efe8; }
.avatar.overlap { margin-left: -10px; }

/* ---- Sliders / progress: recessed grooves with glossy fills / knobs ---- */
.slider-head { display: flex; flex-direction: row; align-items: center; justify-content: space-between;
    height: 18px; margin-bottom: 10px; }
.slider-name { height: 16px; color: #6b6658; font-size: 13px; }
.slider-val { height: 16px; color: #2f6fd6; font-size: 13px; font-weight: 700; text-align: right; }
.slider { position: relative; height: 24px; margin-bottom: 24px; }
.slider-track { position: absolute; left: 0; right: 0; top: 9px; height: 6px;
    border-radius: 3px; border: 1px solid #a9a496;
    background-image: linear-gradient(#c9c5bb, #ddd9d0);
    box-shadow: inset 0 1px 2px rgba(0,0,0,0.3); }
.slider-fill { position: absolute; left: 0; top: 9px; width: 55%; height: 6px;
    border-radius: 3px; border: 1px solid #2560bf;
    background-image: linear-gradient(#6ba8ef, #2f6fd6);
    box-shadow: inset 0 1px 0 rgba(255,255,255,0.4); }
.slider-knob { position: absolute; left: 53%; top: 1px; width: 22px; height: 22px;
    border-radius: 11px; border: 1px solid #b8b3a4;
    background-image: linear-gradient(#ffffff, #dedacf);
    box-shadow: inset 0 1px 0 rgba(255,255,255,0.9), 0 2px 3px rgba(0,0,0,0.4); }
.progress-label { height: 18px; margin-bottom: 8px; color: #6b6658; font-size: 13px; }
.progress { height: 8px; border-radius: 4px; border: 1px solid #a9a496; margin-bottom: 20px;
    background-image: linear-gradient(#c9c5bb, #ddd9d0);
    box-shadow: inset 0 1px 2px rgba(0,0,0,0.3); }
.progress-fill { width: 72%; height: 8px; border-radius: 4px; border: 1px solid #2560bf;
    background-image: linear-gradient(#6ba8ef, #2f6fd6);
    box-shadow: inset 0 1px 0 rgba(255,255,255,0.4); }
.spinner-row { display: flex; flex-direction: row; align-items: center; height: 28px; }
.spinner { width: 28px; height: 28px; border-radius: 14px; margin-right: 14px;
    border: 3px solid #c9c5bb; border-top-color: #2f6fd6; }
.spinner-text { height: 18px; color: #6b6658; font-size: 13px; }

/* ---- Data table: glossy header, ruled rows ---- */
.table { border: 1px solid #a9a496; border-radius: 10px; }
.thead { display: grid; grid-template-columns: 1.6fr 1fr 1fr 0.8fr;
    padding: 14px 18px; border-radius: 10px 10px 0 0;
    background-image: linear-gradient(#e8e6df, #d6d2c6);
    box-shadow: inset 0 1px 0 rgba(255,255,255,0.7); }
.th { height: 16px; color: #4a4636; font-size: 12px; font-weight: 700; }
.th-right { text-align: right; }
.trow { display: grid; grid-template-columns: 1.6fr 1fr 1fr 0.8fr; align-items: center;
    padding: 14px 18px; border-top: 1px solid #cfcabb; }
.td { height: 20px; color: #3a362e; font-size: 14px; }
.td-name { font-weight: 700; }
.td-muted { color: #6b6658; }
.td-mono { color: #3a362e; }
.td-right { text-align: right; }

/* ---- Feedback ---- */
.feedback-row { display: flex; flex-direction: row; align-items: center; height: 40px; }
";
