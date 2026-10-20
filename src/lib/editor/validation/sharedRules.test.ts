import { expect, it } from "vitest";
import cases from "@shared/validation-cases.json";
import { evaluateSharedRules, finiteDecimal } from "./sharedRules";
import { evaluateComboRules } from "./comboRules";
import type { GameParameter } from "@/lib/core";
import { buildIniSnapshot } from "../iniSnapshot";

for (const scenario of cases.comboCases) {
  it(`shared semantics: ${scenario.name}`, () => {
    const values = Object.fromEntries(Object.entries(scenario.values).filter((pair): pair is [string,string] => pair[1] !== undefined));
    expect(evaluateSharedRules({ ...scenario, values })).toEqual(scenario.expected);
    const params = Object.entries(values).map(([key,value]) => ({ key,value,file:key.startsWith("sg.") ? "GameUserSettings.ini" : "Engine.ini", section:"SystemSettings", present_in_ini:true,editable:true,known:true,category:"Rendering" } as GameParameter));
    const files: Record<string, Record<string, Record<string,string>>> = {};
    for (const key of scenario.changedKeys) {
      const file = key.startsWith("sg.") ? "GameUserSettings.ini" : "Engine.ini";
      ((files[file] ??= {}).SystemSettings ??= {})[key] = values[key] ?? "1";
    }
    const actual = evaluateComboRules({ panel:"advanced",params,engineEnabled:new Set(),shippedIniKeys:buildIniSnapshot(params), files,gpu:{name:"Fixture",vendor:"unknown",supports_dlss:false,supports_dlss_fg:false,supports_ray_tracing:scenario.rtStatus==="supported",ray_tracing_status:scenario.rtStatus} });
    expect(actual.map(({code,severity,key}) => ({code,severity,key:key ?? null}))).toEqual(scenario.expected);
  });
}
for (const scenario of cases.numberCases) {
  it(`shared decimal: ${JSON.stringify(scenario.raw)}`, () => expect(finiteDecimal(scenario.raw)).toBe(scenario.expected));
}
