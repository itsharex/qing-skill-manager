import { ref } from "vue";
import { invoke } from "@tauri-apps/api/core";
import type { IdeSkill, LocalSkill } from "./types";
import { getErrorMessage } from "./utils";
import type { AppContext } from "./useAppContext";

export function useIdeAdoption(
  ctx: AppContext
) {
  const busy = ref(false);
  const busyText = ref("");

  async function adoptIdeSkill(skill: IdeSkill): Promise<void> {
    busy.value = true;
    busyText.value = ctx.t("messages.adopting");
    try {
      const message = (await invoke("adopt_ide_skill", {
        request: {
          targetPath: skill.path,
          ideLabel: skill.ide
        }
      })) as string;
      ctx.toast.success(message);
      await ctx.scanLocalSkills();
    } catch (err) {
      ctx.toast.error(getErrorMessage(err, ctx.t("errors.adoptFailed")));
    } finally {
      busy.value = false;
      busyText.value = "";
    }
  }

  async function adoptManyIdeSkills(skills: IdeSkill[]): Promise<void> {
    if (skills.length === 0) return;
    busy.value = true;
    busyText.value = ctx.t("messages.adopting");
    let successCount = 0;
    let failCount = 0;
    const failedSkills: string[] = [];

    try {
      for (const skill of skills) {
        try {
          await invoke("adopt_ide_skill", {
            request: {
              targetPath: skill.path,
              ideLabel: skill.ide
            }
          });
          successCount++;
        } catch {
          failCount++;
          failedSkills.push(skill.name);
        }
      }

      if (successCount > 0 && failCount === 0) {
        ctx.toast.success(ctx.t("messages.adoptedCount", { count: successCount }));
      } else if (successCount > 0 && failCount > 0) {
        ctx.toast.error(ctx.t("messages.adoptedPartial", { success: successCount, failed: failCount }));
        if (failedSkills.length > 0) {
          console.warn("[adoptManyIdeSkills] Failed to adopt:", failedSkills);
        }
      } else {
        ctx.toast.error(ctx.t("errors.adoptFailed"));
      }

      const scanResult = await ctx.scanLocalSkills();
      if (!scanResult) {
        console.warn("[adoptManyIdeSkills] scanLocalSkills failed after adopt");
      }
    } finally {
      busy.value = false;
      busyText.value = "";
    }
  }

  async function adoptToRepo(
    path: string,
    localSkills: { value: LocalSkill[] },
    loadSkillPackage: (skillId: string) => Promise<unknown>
  ): Promise<void> {
    const dirName = path.split("/").pop() || "";
    try {
      await invoke("import_local_skill", { request: { sourcePath: path } });
      await ctx.scanLocalSkills();
      // Find the newly imported skill and load its package
      const newSkill = localSkills.value.find((s) => s.name === dirName || s.path.endsWith(`/${dirName}`));
      if (newSkill?.currentVersion) {
        void loadSkillPackage(newSkill.currentVersion.skillId || newSkill.id);
      }
    } catch (err) {
      console.error("Failed to adopt skill:", err);
    }
  }

  async function adoptManyToRepo(targets: Array<{ path: string; ideLabel: string }>): Promise<void> {
    busy.value = true;
    busyText.value = ctx.t("messages.adopting");
    try {
      for (const target of targets) {
        try {
          await invoke("adopt_ide_skill", {
            request: { targetPath: target.path, ideLabel: target.ideLabel || "IDE" }
          });
        } catch (err) {
          console.warn("Failed to adopt skill:", target.path, err);
        }
      }
      await ctx.scanLocalSkills();
    } finally {
      busy.value = false;
      busyText.value = "";
    }
  }

  return {
    busy,
    busyText,
    adoptIdeSkill,
    adoptManyIdeSkills,
    adoptToRepo,
    adoptManyToRepo
  };
}
