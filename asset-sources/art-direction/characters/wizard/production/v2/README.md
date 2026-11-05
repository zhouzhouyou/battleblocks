# 老巫师粗像素小人 · v2

用户最新原文：「改回去，粗像素小人保留」。覆盖v1细密全身重绘。普通恢复原始半身构图、大色块、方块眼、长鼻、白胡须和蓝紫帽；觉醒沿用原有小全身蓝袍、鞋和蓝晶木杖。原始 `assets/wizzard.webp` 与 `assets/awakened/wizard-awakening.webp` 均未修改。

两份 `*-prompt-v2.md` 为内置imagegen实际提示词；两份 `*-source-v2.webp` 为生成结果的无损转换。每张4列6行，上12张待机、下12张施法，1024×1536；普通以旧四帧条带为强制参考，觉醒以旧普通画法与旧觉醒装备轮廓为参考。用户具体画风要求优先于全局E细密制作默认值。

`tools/prepare_wizard_assets.py` 只做导出：alpha阈值160，全部源格以同一比例采样到22×22并注册进24×24逻辑画布；所有姿势的身体轴固定，普通下边缘和觉醒鞋底统一逻辑y=21。输出以16倍最近邻放入384帧，没有插值补动作。原普通首帧直接从旧图无损还原15×20逻辑像素置入，其他帧来自独立绘制的源格。整个图集只使用原七色加两种晶石蓝白色；透明RGB清零、二值alpha。单独的回归测试锁定16×16像素块及原始中立脸，防止再次悄悄细化画风。

每组12个不同像素姿势，四组48帧、6×8图集。普通待机的基线是半身下边缘，不能称为脚底。帧哈希与逻辑边界见 `report.json`。运行时仍是 `assets/roles/wizard.webp`；原始源图和循环评审WebP不进包。导入：

```powershell
& (Get-Content .build-tools/python-path.txt) tools/prepare_character_animations.py --role wizard
& (Get-Content .build-tools/python-path.txt) tools/prepare_character_animations.py --registry-only
```

待机12×60ms；施法时长ms为60,80,90,90,120,180,280,100,100,100,100,100。普通用头帽倾动和下边缘的简洁手袖色块表达施法，觉醒用杖和手势；第5帧出手、第7帧引导、第8帧命中，随后0.5s收势。命中同步逻辑未变。当前视觉、实战与验证见 [v2验收](../../../../../../analysis/issue27-wizard-animation/v2/README.md)。
