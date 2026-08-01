#![doc = include_str!("flat_theme.doc/_module.md")]

#[doc = include_str!("flat_theme.doc/FLAT_CSS.md")]
pub(super) const FLAT_CSS: &str = r"
* { box-sizing: border-box; }
#app { display: flex; flex-direction: column; background-color: #f1f5f9; }

/* ---- Switcher bar ---- */
.kit-switcher { display: flex; flex-direction: row; align-items: center; height: 60px;
    padding-left: 18px; padding-right: 18px; background-color: #ffffff;
    border-bottom: 1px solid #e2e8f0; }
.kit-brand { display: flex; flex-direction: row; align-items: center; width: 60px; height: 60px; }
.kit-logo { width: 11px; height: 11px; border-radius: 4px; background-color: #2563eb; }
.kit-brand-name { width: 40px; height: 16px; margin-left: 8px; color: #0f172a;
    font-size: 13px; font-weight: 700; }
.kit-tabs { display: flex; flex-direction: row; }
.kit-tab { display: flex; flex-direction: column; align-items: center; justify-content: center;
    width: 108px; height: 60px; }
.kit-tab-label { height: 16px; color: #94a3b8; font-size: 12px; font-weight: 600; text-align: center; }
.kit-tab.active .kit-tab-label { color: #0f172a; }
.kit-underline { width: 60px; height: 2px; border-radius: 2px; background-color: #2563eb; margin-top: 6px; }

/* ---- Gallery frame ---- */
.m3-page { display: flex; flex-direction: column; padding: 34px 40px 60px; }
.m3-head { display: flex; flex-direction: column; height: 108px; margin-bottom: 26px; }
.m3-eyebrow { height: 16px; color: #94a3b8; font-size: 11px; font-weight: 600; }
.m3-title { height: 48px; color: #0f172a; font-size: 38px; font-weight: 700; }
.m3-sub { height: 22px; color: #64748b; font-size: 15px; }

.gallery-grid { display: flex; flex-direction: column; }
.row { display: flex; flex-direction: row; align-items: stretch; margin-bottom: 24px; }
.row.r1 { height: 232px; }
.row.r2 { height: 336px; }
.row.r3 { height: 420px; }
.row.r4 { height: 252px; }

/* Cards: white, 1px slate border, medium radius, no shadow. */
.card { background-color: #ffffff; border-radius: 10px; padding: 22px;
    border: 1px solid #e2e8f0; }
.card.col { width: 588px; margin-right: 24px; }
.card.full { width: 1200px; margin-bottom: 24px; }
.card.full.table-c { height: 288px; }
.card.full.feedback-c { height: 132px; }
.card-label { height: 16px; margin-bottom: 16px; color: #94a3b8; font-size: 11px; font-weight: 600; }

/* ---- Buttons: solid blue primary, white+border secondary, 8px radius ---- */
.btn-row { display: flex; flex-direction: row; align-items: center; height: 56px; }
.btn { display: flex; flex-direction: row; align-items: center; justify-content: center;
    height: 40px; border-radius: 8px; padding-left: 20px; padding-right: 20px; margin-right: 10px; }
.btn-label { height: 18px; font-size: 14px; font-weight: 600; text-align: center; }
.btn-filled { width: 110px; background-color: #2563eb; }
.btn-filled .btn-label { color: #ffffff; }
.btn-tonal { width: 104px; background-color: #eff6ff; border: 1px solid #bfdbfe; }
.btn-tonal .btn-label { color: #1d4ed8; }
.btn-outlined { width: 128px; background-color: #ffffff; border: 1px solid #cbd5e1; }
.btn-outlined .btn-label { color: #334155; }
.btn-text { width: 92px; padding-left: 16px; padding-right: 16px; background-color: #ffffff; }
.btn-text .btn-label { color: #2563eb; }
.btn-dialog { width: 152px; }
.btn-elevated { width: 132px; background-color: #ffffff; border: 1px solid #cbd5e1; }
.btn-elevated .btn-label { color: #334155; }
.btn-disabled { width: 128px; background-color: #f1f5f9; border: 1px solid #e2e8f0; }
.btn-disabled .btn-label { color: #cbd5e1; }
/* FAB → the square icon button of the SaaS set. */
.fab { display: flex; flex-direction: row; align-items: center; justify-content: center;
    width: 56px; height: 56px; border-radius: 8px; background-color: #ffffff; border: 1px solid #cbd5e1; }
.fab-glyph { height: 30px; color: #475569; font-size: 26px; font-weight: 400; text-align: center; }
.btn-sm { height: 38px; }
.btn-wide { width: 168px; }

/* ---- Selection ---- */
.sel-cols { display: flex; flex-direction: row; }
.sel-col { display: flex; flex-direction: column; width: 168px; }
.sel-item { display: flex; flex-direction: row; align-items: center; height: 34px; }
.sel-col .sel-item { margin-bottom: 14px; }
.sel-col.wide-gap .sel-item { margin-bottom: 18px; height: 32px; }
.sel-text { height: 20px; margin-left: 11px; color: #0f172a; font-size: 14px; }
.checkbox-on { display: flex; align-items: center; justify-content: center;
    width: 18px; height: 18px; border-radius: 5px; background-color: #2563eb; }
.check-glyph { height: 14px; color: #ffffff; font-size: 12px; text-align: center; }
.checkbox-off { width: 18px; height: 18px; border-radius: 5px; border: 1px solid #cbd5e1; }
.checkbox-disabled { width: 18px; height: 18px; border-radius: 5px; border: 1px solid #e2e8f0;
    background-color: #f1f5f9; }
.radio-on { display: flex; align-items: center; justify-content: center;
    width: 20px; height: 20px; border-radius: 10px; border: 1px solid #cbd5e1; }
.radio-dot { width: 10px; height: 10px; border-radius: 5px; background-color: #2563eb; }
.radio-off { width: 20px; height: 20px; border-radius: 10px; border: 1px solid #cbd5e1; }
.toggle-on { display: flex; flex-direction: row; align-items: center; justify-content: flex-end;
    width: 44px; height: 24px; border-radius: 12px; background-color: #2563eb; padding: 3px; }
.toggle-knob { width: 18px; height: 18px; border-radius: 9px; background-color: #ffffff; }
.toggle-off { display: flex; flex-direction: row; align-items: center; justify-content: flex-start;
    width: 44px; height: 24px; border-radius: 12px; background-color: #cbd5e1; padding: 3px; }
.toggle-knob-off { width: 18px; height: 18px; border-radius: 9px; background-color: #ffffff; }

/* ---- Text fields ---- */
.field-col { display: flex; flex-direction: column; }
.field { margin-bottom: 16px; }
.field-filled { background-color: #ffffff; border-radius: 8px; border: 1px solid #cbd5e1; padding: 9px 12px; }
.field-outlined { background-color: #ffffff; border: 1px solid #cbd5e1; border-radius: 8px; padding: 9px 12px; }
.field-error { background-color: #fef2f2; border: 1px solid #dc2626; border-radius: 8px; padding: 9px 12px; }
.field-label-primary { height: 14px; margin-bottom: 4px; color: #475569; font-size: 11px; font-weight: 500; }
.field-label { height: 14px; margin-bottom: 4px; color: #475569; font-size: 11px; font-weight: 500; }
.field-label-error { height: 14px; margin-bottom: 4px; color: #dc2626; font-size: 11px; font-weight: 500; }
.field-value { height: 20px; color: #0f172a; font-size: 15px; }
.field-placeholder { height: 20px; color: #94a3b8; font-size: 15px; }
.field-error-text { height: 14px; margin-top: 5px; color: #dc2626; font-size: 11px; }

/* ---- Menu / dropdown ---- */
.select-box { display: flex; flex-direction: row; align-items: center; justify-content: space-between;
    width: 260px; height: 44px; background-color: #ffffff; border: 1px solid #cbd5e1; border-radius: 8px; padding: 0 14px; }
.select-value { height: 20px; color: #0f172a; font-size: 15px; }
.select-chevron { width: 16px; height: 20px; color: #94a3b8; font-size: 14px; text-align: right; }
.menu-panel { display: flex; flex-direction: column; width: 260px; height: 136px; margin-top: 8px;
    background-color: #ffffff; border: 1px solid #e2e8f0; border-radius: 8px; padding-top: 6px; padding-bottom: 6px;
    box-shadow: 0 10px 28px rgba(15,23,42,0.12); }
.menu-item { display: flex; flex-direction: row; align-items: center; justify-content: space-between;
    height: 40px; padding: 0 14px; }
.menu-item.selected { background-color: #f1f5f9; border-radius: 6px; }
.menu-text { height: 20px; color: #0f172a; font-size: 15px; }
.menu-check { width: 16px; height: 20px; color: #2563eb; font-size: 14px; text-align: right; }
.chips-row { display: flex; flex-direction: row; align-items: center; height: 32px; margin-top: 16px; }
.chip { display: flex; flex-direction: row; align-items: center; justify-content: center; height: 32px;
    border-radius: 7px; padding-left: 12px; padding-right: 12px; margin-right: 8px; }
.chip-filled { width: 112px; background-color: #f1f5f9; border: 1px solid #e2e8f0; }
.chip-filled .chip-text { color: #334155; }
.chip-check { height: 16px; margin-right: 6px; color: #64748b; font-size: 12px; }
.chip-outlined { width: 104px; background-color: #f1f5f9; border: 1px solid #e2e8f0; }
.chip-outlined .chip-text { color: #334155; }
.chip-tonal { background-color: #f1f5f9; border: 1px solid #e2e8f0; }
.chip-tonal .chip-text { color: #334155; }
.chip-wide { width: 172px; }
.chip-text { height: 18px; font-size: 14px; font-weight: 600; }

/* ---- Card & list ---- */
.mini-card { display: flex; flex-direction: column; height: 208px; background-color: #ffffff;
    border: 1px solid #e2e8f0; border-radius: 10px; }
.mini-cover { display: flex; align-items: center; justify-content: center; height: 84px;
    border-radius: 10px 10px 0 0; background-color: #e2e8f0; }
.mini-cover-text { height: 16px; color: #94a3b8; font-size: 11px; text-align: center; }
.mini-body { display: flex; flex-direction: column; height: 124px; padding: 16px; }
.mini-title { height: 22px; color: #0f172a; font-size: 16px; font-weight: 700; }
.mini-sub { height: 18px; margin-top: 4px; color: #64748b; font-size: 13px; }
.mini-actions { display: flex; flex-direction: row; align-items: center; height: 38px; margin-top: 14px; }
.people { display: flex; flex-direction: column; height: 100px; margin-top: 14px; }
.person { display: flex; flex-direction: row; align-items: center; height: 50px; }
.person-meta { display: flex; flex-direction: column; justify-content: center; height: 50px;
    margin-left: 13px; width: 420px; }
.person-name { height: 18px; color: #0f172a; font-size: 14px; }
.person-email { height: 16px; margin-top: 2px; color: #64748b; font-size: 12px; }
.person-menu { width: 24px; height: 20px; color: #94a3b8; font-size: 18px; text-align: right; }
.avatar { display: flex; align-items: center; justify-content: center;
    width: 36px; height: 36px; border-radius: 9px; }
.avatar-text { height: 16px; font-size: 13px; font-weight: 700; text-align: center; color: #ffffff; }
.avatar-ak { background-color: #2563eb; }
.avatar-mr { background-color: #0891b2; }
.avatar-jp { background-color: #7c3aed; }
.avatar-more { background-color: #f1f5f9; }
.avatar-more .avatar-text { color: #64748b; }

/* ---- Navigation ---- */
.breadcrumb { display: flex; flex-direction: row; align-items: center; height: 20px; margin-bottom: 18px; }
.crumb { height: 18px; margin-right: 8px; color: #64748b; font-size: 13px; }
.crumb-sep { height: 18px; margin-right: 8px; color: #cbd5e1; font-size: 13px; }
.crumb-active { height: 18px; color: #0f172a; font-size: 13px; font-weight: 600; }
.nav-tabs { display: flex; flex-direction: row; height: 42px;
    border-bottom: 2px solid #e2e8f0; margin-bottom: 14px; }
.nav-tab { display: flex; flex-direction: column; align-items: center; justify-content: flex-end;
    width: 96px; }
.nav-tab-label { height: 24px; color: #64748b; font-size: 14px; font-weight: 600; text-align: center; }
.nav-tab-label.active { color: #2563eb; }
.nav-tab-underline { width: 96px; height: 2px; background-color: #2563eb; }
.nav-list { display: flex; flex-direction: column; width: 260px; }
.nav-item { display: flex; flex-direction: row; align-items: center; height: 40px;
    border-radius: 7px; padding-left: 12px; padding-right: 12px; margin-bottom: 4px; }
.nav-item.active { background-color: #eff6ff; border-left: 3px solid #2563eb; }
.nav-item-text { height: 20px; margin-left: 10px; color: #475569; font-size: 14px; }
.nav-item.active .nav-item-text { color: #1d4ed8; font-weight: 600; }
.dot-on { width: 6px; height: 6px; border-radius: 2px; background-color: #2563eb; }
.dot-off { width: 6px; height: 6px; border-radius: 2px; background-color: #cbd5e1; }

/* ---- Badges / chips / avatars: solid pastel status pills, 6px radius ---- */
.badge-row { display: flex; flex-direction: row; align-items: center; height: 26px; margin-bottom: 22px; }
.badge { display: flex; align-items: center; justify-content: center; height: 24px;
    border-radius: 6px; margin-right: 8px; }
.badge-text { height: 16px; font-size: 12px; font-weight: 600; text-align: center; }
.badge-success { width: 76px; background-color: #dcfce7; }
.badge-success .badge-text { color: #166534; }
.badge-warn { width: 84px; background-color: #fef3c7; }
.badge-warn .badge-text { color: #92400e; }
.badge-error { width: 66px; background-color: #fee2e2; }
.badge-error .badge-text { color: #991b1b; }
.badge-neutral { width: 64px; background-color: #f1f5f9; }
.badge-neutral .badge-text { color: #64748b; }
.badge-count { display: flex; align-items: center; justify-content: center;
    width: 24px; height: 20px; border-radius: 6px; background-color: #2563eb; }
.badge-count-text { height: 14px; color: #ffffff; font-size: 11px; font-weight: 700; text-align: center; }
.avatar-stack { display: flex; flex-direction: row; align-items: center; height: 40px; }
.avatar.stack { border: 2px solid #ffffff; }
.avatar.overlap { margin-left: -9px; }

/* ---- Sliders / progress ---- */
.slider-head { display: flex; flex-direction: row; align-items: center; justify-content: space-between;
    height: 18px; margin-bottom: 10px; }
.slider-name { height: 16px; color: #64748b; font-size: 13px; }
.slider-val { height: 16px; color: #2563eb; font-size: 13px; font-weight: 600; text-align: right; }
.slider { position: relative; height: 24px; margin-bottom: 24px; }
.slider-track { position: absolute; left: 0; right: 0; top: 9px; height: 6px;
    border-radius: 3px; background-color: #e2e8f0; }
.slider-fill { position: absolute; left: 0; top: 9px; width: 55%; height: 6px;
    border-radius: 3px; background-color: #2563eb; }
.slider-knob { position: absolute; left: 53%; top: 3px; width: 18px; height: 18px;
    border-radius: 9px; background-color: #ffffff; border: 2px solid #2563eb; }
.progress-label { height: 18px; margin-bottom: 8px; color: #64748b; font-size: 13px; }
.progress { height: 8px; border-radius: 4px; background-color: #e2e8f0; margin-bottom: 20px; }
.progress-fill { width: 72%; height: 8px; border-radius: 4px; background-color: #2563eb; }
.spinner-row { display: flex; flex-direction: row; align-items: center; height: 28px; }
.spinner { width: 24px; height: 24px; border-radius: 12px; margin-right: 14px;
    border: 3px solid #e2e8f0; border-top-color: #2563eb; }
.spinner-text { height: 18px; color: #64748b; font-size: 13px; }

/* ---- Data table ---- */
.table { border: 1px solid #e2e8f0; border-radius: 8px; }
.thead { display: grid; grid-template-columns: 1.6fr 1fr 1fr 0.8fr;
    background-color: #f8fafc; padding: 12px 16px; border-radius: 8px 8px 0 0; }
.th { height: 16px; color: #94a3b8; font-size: 12px; font-weight: 700; }
.th-right { text-align: right; }
.trow { display: grid; grid-template-columns: 1.6fr 1fr 1fr 0.8fr; align-items: center;
    padding: 12px 16px; border-top: 1px solid #f1f5f9; }
.td { height: 20px; color: #0f172a; font-size: 14px; }
.td-name { font-weight: 600; }
.td-muted { color: #64748b; }
.td-mono { color: #0f172a; }
.td-right { text-align: right; }

/* ---- Feedback ---- */
.feedback-row { display: flex; flex-direction: row; align-items: center; height: 40px; }
";
