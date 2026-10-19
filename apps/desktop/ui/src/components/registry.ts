/** Source/consumer mapping for the stable IDs in the prototype catalogue.
 * Imported by DEV gallery only; documentation owns the specification.
 * Registration and existing source paths do not claim full visual/native acceptance.
 */
export interface ComponentRegistration {
 id:string;name:string;section:string;kind:'component'|'recipe'|'platform'|'prototype_only';
 implementations:string[];consumers:string[];gallery:string|null;
 specification:{document:string;componentId:string};
 verification:'pending_full_matrix';reason?:string;metrics?:Record<string,unknown>
}
const entries:Array<Omit<ComponentRegistration,'specification'|'verification'>> = [
 {"id":"brand.logo","name":"Logo与标识","section":"brand","kind":"platform","implementations":["components/BrandMark.vue"],"consumers":["components/AppSidebar.vue"],"gallery":"brand"},
 {"id":"brand.app-macos","name":"macOS应用图标","section":"app-icons","kind":"platform","implementations":["../tauri/icons/icon.icns"],"consumers":["../tauri/tauri.conf.json"],"gallery":"brand"},
 {"id":"brand.app-windows","name":"Windows应用图标","section":"app-icons","kind":"platform","implementations":["../tauri/icons/windows/icon.ico"],"consumers":["../tauri/tauri.conf.json"],"gallery":"brand"},
 {"id":"brand.tray-macos","name":"macOS托盘图标","section":"tray-icons","kind":"platform","implementations":["../tauri/src/infrastructure/tray_controller.rs"],"consumers":["../tauri/src/lib.rs"],"gallery":"brand"},
 {"id":"brand.tray-windows","name":"Windows托盘图标","section":"tray-icons","kind":"platform","implementations":["../tauri/src/infrastructure/windows_tray_icon.rs"],"consumers":["../tauri/src/infrastructure/tray_controller.rs"],"gallery":"brand"},
 {"id":"type.scale","name":"字号与字体","section":"typography","kind":"recipe","implementations":["styles/tokens.css"],"consumers":["routes/SettingsRoute.vue"],"gallery":"typography"},
 {"id":"content.markdown","name":"代码与富文本","section":"typography","kind":"component","implementations":["components/patterns/MarkdownContent.vue"],"consumers":["features/extensions/components/ExtensionDetailModal.vue"],"gallery":"typography"},
 {"id":"surface.recipe","name":"语义材质","section":"materials","kind":"recipe","implementations":["styles/materials.css"],"consumers":["App.vue"],"gallery":"materials"},
 {"id":"appearance.effects","name":"画质与主题","section":"materials","kind":"recipe","implementations":["app/appearance/effects.ts"],"consumers":["App.vue"],"gallery":"materials"},
 {"id":"icon.scale","name":"图标与点击区域","section":"icons-spacing","kind":"recipe","implementations":["styles/tokens.css"],"consumers":["components/AppTopbar.vue"],"gallery":"typography"},
 {"id":"layout.stack","name":"纵向排列","section":"layout","kind":"component","implementations":["components/layout/Stack.vue"],"consumers":["dev/component-gallery/GalleryApp.vue"],"gallery":"layout"},
 {"id":"layout.inline","name":"横向排列","section":"layout","kind":"component","implementations":["components/layout/Inline.vue"],"consumers":["dev/component-gallery/GalleryApp.vue"],"gallery":"layout"},
 {"id":"layout.split","name":"分栏与响应式","section":"layout","kind":"recipe","implementations":["styles/base.css"],"consumers":["routes/ModelsRoute.vue"],"gallery":"layout"},
 {"id":"navigation.anchor","name":"锚点与固定条","section":"layout","kind":"component","implementations":["components/layout/StickyTabs.vue"],"consumers":["routes/ModelsRoute.vue"],"gallery":"tabs"},
 {"id":"button.text","name":"文字按钮","section":"buttons","kind":"component","implementations":["components/ui/UiButton.vue"],"consumers":["routes/SettingsRoute.vue","routes/OverviewRoute.vue","features/models/ChannelsView.vue"],"gallery":"buttons","metrics":{"sizes":{"sm":28,"md":34,"lg":40},"variants":["default","primary","ghost","danger"],"states":["default","hover","active","focus","disabled","loading"]}},
 {"id":"button.icon","name":"图标按钮","section":"buttons","kind":"component","implementations":["components/ui/UiIconButton.vue"],"consumers":["components/AppTopbar.vue","features/notifications/components/NotificationItem.vue"],"gallery":"typography"},
 {"id":"field.input","name":"输入字段","section":"forms","kind":"recipe","implementations":["styles/base.css"],"consumers":["features/models/ChannelEditor.vue"],"gallery":"native-fields"},
 {"id":"field.textarea","name":"多行输入","section":"forms","kind":"recipe","implementations":["styles/base.css"],"consumers":["features/models/AgentPanel.vue"],"gallery":"native-fields"},
 {"id":"choice.checkbox","name":"复选/半选","section":"forms","kind":"recipe","implementations":["styles/base.css"],"consumers":["features/models/ModelPicker.vue"],"gallery":"native-fields"},
 {"id":"choice.radio","name":"单选","section":"forms","kind":"recipe","implementations":["styles/base.css"],"consumers":["features/models/BatchAgentDialog.vue"],"gallery":"native-fields"},
 {"id":"choice.switch","name":"开关","section":"forms","kind":"component","implementations":["components/ui/UiSwitch.vue"],"consumers":["routes/SettingsRoute.vue"],"gallery":"switches","metrics":{"track":"38 × 22","hitHeight":"28","states":["off","on","focus","disabled","busy"]}},
 {"id":"choice.slider","name":"滑杆","section":"forms","kind":"component","implementations":["components/ui/UiDiscreteSlider.vue"],"consumers":["features/models/ModelChoiceOptions.vue"],"gallery":"fields"},
 {"id":"choice.select","name":"选择与下拉","section":"menus-tabs","kind":"component","implementations":["components/ui/UiSelectField.vue","components/ui/UiSelectMenu.vue"],"consumers":["routes/SettingsRoute.vue","routes/ModelsRoute.vue"],"gallery":"fields"},
 {"id":"menu.actions","name":"动作菜单","section":"menus-tabs","kind":"component","implementations":["components/ui/UiSelectMenu.vue"],"consumers":["routes/SettingsRoute.vue"],"gallery":"action-menu"},
 {"id":"navigation.tabs","name":"Tab分区","section":"menus-tabs","kind":"component","implementations":["components/layout/StickyTabs.vue"],"consumers":["routes/SettingsRoute.vue","routes/ExtensionsRoute.vue","routes/LogsRoute.vue","routes/ModelsRoute.vue","routes/SyncRoute.vue"],"gallery":"tabs"},
 {"id":"data.table","name":"数据表","section":"data","kind":"recipe","implementations":["styles/base.css"],"consumers":["features/models/ChannelsView.vue"],"gallery":"data"},
 {"id":"data.tree","name":"树表","section":"data","kind":"component","implementations":["components/ui/UiTreeTable.vue"],"consumers":["features/backup/BackupFiles.vue","features/sync/ObjectScopeTree.vue"],"gallery":"data"},
 {"id":"data.status","name":"状态标签","section":"data","kind":"recipe","implementations":["styles/base.css"],"consumers":["features/models/ChannelsView.vue"],"gallery":"data"},
 {"id":"overlay.dialog","name":"Dialog/Modal","section":"overlays","kind":"component","implementations":["components/ui/UiDialog.vue"],"consumers":["features/sync/ModelTransfer.vue"],"gallery":"fields"},
 {"id":"overlay.annotation","name":"原型注释","section":"overlays","kind":"prototype_only","implementations":[],"consumers":[],"gallery":null,"reason":"原型说明只在文档工作台存在，正式软件不导入"},
 {"id":"feedback.local","name":"空态/校验/加载","section":"feedback","kind":"recipe","implementations":["styles/base.css"],"consumers":["features/models/ChannelsView.vue"],"gallery":"data"},
 {"id":"feedback.steps","name":"步骤与进度","section":"feedback","kind":"recipe","implementations":["styles/base.css"],"consumers":["components/TaskProgressHost.vue"],"gallery":"feedback"},
 {"id":"model.selection","name":"模型选择器","section":"model-picker","kind":"component","implementations":["features/models/ModelPicker.vue","features/models/ModelChoiceOptions.vue"],"consumers":["features/models/AgentExecutionField.vue","features/models/ChannelsView.vue"],"gallery":"model-picker"},
 {"id":"node.selection","name":"节点与检测","section":"node-picker","kind":"component","implementations":["features/models/NodePicker.vue","features/models/useNodeMeasurements.ts"],"consumers":["features/models/ChannelEditor.vue"],"gallery":"nodes"},
 {"id":"feedback.toast","name":"短结果反馈","section":"notifications","kind":"component","implementations":["components/patterns/FeedbackToast.vue","components/ToastHost.vue"],"consumers":["App.vue"],"gallery":"feedback"},
 {"id":"feedback.task","name":"任务卡","section":"notifications","kind":"component","implementations":["components/patterns/TaskProgressCard.vue","components/TaskProgressHost.vue"],"consumers":["App.vue"],"gallery":"feedback"},
 {"id":"feedback.notification","name":"通知中心条目","section":"notifications","kind":"component","implementations":["features/notifications/components/NotificationItem.vue"],"consumers":["features/notifications/components/NotificationCenter.vue"],"gallery":"feedback"},
 {"id":"platform.caption","name":"Windows标题栏","section":"windows","kind":"platform","implementations":["../tauri/src/infrastructure/window_appearance.rs"],"consumers":["../tauri/src/lib.rs"],"gallery":"platform-boundary"},
 {"id":"platform.tray","name":"托盘菜单","section":"tray","kind":"platform","implementations":["../tauri/src/modules/tray/mod.rs","../tauri/src/infrastructure/tray_controller.rs"],"consumers":["../tauri/src/lib.rs"],"gallery":"platform-boundary"},
 {"id":"pattern.setting","name":"设置行","section":"examples","kind":"component","implementations":["components/patterns/SettingRow.vue"],"consumers":["routes/SettingsRoute.vue"],"gallery":"settings"},
 {"id":"pattern.card","name":"卡片","section":"examples","kind":"component","implementations":["components/ui/UiCard.vue"],"consumers":["routes/OverviewRoute.vue","routes/ExtensionsRoute.vue"],"gallery":"cards"},
 {"id":"pattern.card-header","name":"卡片标题","section":"examples","kind":"component","implementations":["components/ui/UiCardHeader.vue"],"consumers":["routes/OverviewRoute.vue","routes/ExtensionsRoute.vue"],"gallery":"cards"},
 {"id":"overlay.modal-host","name":"应用Modal宿主","section":"overlays","kind":"component","implementations":["components/AppModal.vue","components/patterns/ModalContent.vue"],"consumers":["App.vue"],"gallery":"modal-content"},
]
export const componentRegistry:ReadonlyArray<ComponentRegistration> = entries.map(entry=>({
 ...entry,specification:{document:'UI规范§34.1 / 组件与样式专题',componentId:entry.id},verification:'pending_full_matrix',
}))
