import { mount } from "@vue/test-utils"
import { describe, expect, it } from "vitest"
import UiTreeTable from "../src/components/ui/UiTreeTable.vue"
import type { TreeTableNode } from "../src/components/ui/treeTable"

function nodes(): TreeTableNode[] {
  return [{ id: "root", label: "backups", children: [{ id: "year", label: "2026", children: [{ id: "month", label: "10", children: [{ id: "backup", label: "升级备份", selectable: true, selected: "mixed", children: [{ id: "file", label: "preferences.json", selectable: true, disabled: true }] }] }] }] }]
}

describe("shared tree table", () => {
  it("shows month but not its children initially; expansion never selects", async () => {
    const wrapper = mount(UiTreeTable, { props: { label: "备份文件", nodes: nodes() } })
    expect(wrapper.findAll("tbody tr").map(row => row.attributes("data-node-id"))).toEqual(["root", "year", "month"])
    await wrapper.get('[aria-label="展开 10"]').trigger("click")
    expect(wrapper.find('[data-node-id="backup"]').exists()).toBe(true)
    expect(wrapper.emitted("select")).toBeUndefined()
    const checkbox = wrapper.get('[aria-label="选择 升级备份"]')
    expect((checkbox.element as HTMLInputElement).indeterminate).toBe(true)
    await checkbox.setValue(true)
    expect(wrapper.emitted("select")).toEqual([["backup", true]])
    expect(wrapper.find('[data-node-id="file"]').exists()).toBe(false)
  })

  it("preserves expansion and focus across refreshed data and returns focus on collapse", async () => {
    const wrapper = mount(UiTreeTable, { attachTo: document.body, props: { label: "备份文件", nodes: nodes() } })
    await wrapper.get('[aria-label="展开 10"]').trigger("keydown", { key: "ArrowRight" })
    await wrapper.get('[aria-label="展开 升级备份"]').trigger("keydown", { key: "ArrowRight" })
    const file = wrapper.get('[aria-label="preferences.json"]').element as HTMLButtonElement
    file.focus()
    await wrapper.setProps({ nodes: nodes() })
    expect(document.activeElement).toBe(file)
    await wrapper.get('[aria-label="收起 10"]').trigger("click")
    await wrapper.vm.$nextTick()
    expect(document.activeElement?.getAttribute("aria-label")).toBe("展开 10")
    wrapper.unmount()
  })

  it("keeps disabled selection separate from keyboard navigation and supports columns", async () => {
    const wrapper = mount(UiTreeTable, { attachTo: document.body, props: { label: "备份文件", nodes: nodes(), initialExpandedDepth: 8, columns: [{ key: "purpose", label: "用途" }] }, slots: { "cell-purpose": "恢复管理器偏好" } })
    expect(wrapper.get('[aria-label="选择 preferences.json"]').attributes("disabled")).toBeDefined()
    await wrapper.get('[aria-label="preferences.json"]').trigger("keydown", { key: "Home" })
    await wrapper.vm.$nextTick()
    expect(document.activeElement?.getAttribute("aria-label")).toBe("收起 backups")
    expect(wrapper.get("table").attributes("aria-label")).toBe("备份文件")
    expect(wrapper.findAll("thead th")).toHaveLength(2)
    expect(wrapper.get("tbody td").text()).toBe("恢复管理器偏好")
    wrapper.unmount()
  })

  it("restores visible leaf focus after async refresh without stealing outside focus", async () => {
    const wrapper = mount(UiTreeTable, { attachTo: document.body, props: { label: "备份文件", nodes: nodes(), initialExpandedDepth: 8 } })
    const leaf = wrapper.get('[aria-label="preferences.json"]')
    expect(leaf.text()).toBe("preferences.json")
    ;(leaf.element as HTMLButtonElement).focus()
    await wrapper.setProps({ loading: true })
    await wrapper.setProps({ nodes: nodes(), loading: false })
    await wrapper.vm.$nextTick()
    expect(document.activeElement?.getAttribute("aria-label")).toBe("preferences.json")
    await wrapper.setProps({ loading: true })
    const outside = document.createElement("button")
    document.body.append(outside)
    outside.focus()
    await wrapper.setProps({ nodes: nodes(), loading: false })
    await wrapper.vm.$nextTick()
    expect(document.activeElement).toBe(outside)
    outside.remove()
    wrapper.unmount()
  })

  it("renders empty/loading/error without stale actionable rows", async () => {
    const wrapper = mount(UiTreeTable, { props: { label: "备份文件", nodes: [] } })
    expect(wrapper.text()).toContain("暂无内容")
    await wrapper.setProps({ nodes: nodes(), loading: true })
    expect(wrapper.find('[data-node-id="root"]').exists()).toBe(false)
    expect(wrapper.text()).toContain("正在读取")
    await wrapper.setProps({ loading: false, error: "读取失败" })
    expect(wrapper.get('[role="alert"]').text()).toBe("读取失败")
    await wrapper.get("button").trigger("click")
    expect(wrapper.emitted("retry")).toHaveLength(1)
  })
})
