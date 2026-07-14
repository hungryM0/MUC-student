import { describe, expect, it } from "vitest";
import type { AccountDto } from "@/lib/muc";
import { sortAccounts, type SortOption } from "./account-pool-section";

function account(
  id: string,
  remarkName: string,
  isUnlimitedPlan: boolean,
  isCurrentOnline = false,
): AccountDto {
  return {
    id,
    remarkName,
    username: id,
    snapshot: {
      isUnlimitedPlan,
      usedTrafficText: "1.00GB",
      packageAvailableText: "0GB",
      statusText: "已同步",
    } as AccountDto["snapshot"],
    isCurrentOnline,
    canLogoutLocalDevice: false,
  };
}

describe("账号池排序", () => {
  it.each<SortOption>(["remaining", "recent", "name"])(
    "在 %s 排序下把不限流量账号置顶",
    (sortBy) => {
      const accounts = [
        account("regular-online", "普通在线", false, true),
        account("unlimited-b", "无限 B", true),
        account("regular", "普通账号", false),
        account("unlimited-a", "无限 A", true),
      ];

      const result = sortAccounts(
        accounts,
        sortBy,
        { "regular-online": 100, regular: 90 },
        "regular-online",
      );

      expect(
        result
          .slice(0, 2)
          .map((item) => item.id)
          .sort(),
      ).toEqual(["unlimited-a", "unlimited-b"]);
    },
  );

  it("剩余量排序把未知流量账号放在已知账号之后", () => {
    const unknown = account("unknown", "未知", false);
    if (unknown.snapshot) {
      unknown.snapshot.statusText = "查询失败";
    }
    const accounts = [unknown, account("known", "已知", false)];

    const result = sortAccounts(accounts, "remaining", {}, "");

    expect(result.map((item) => item.id)).toEqual(["known", "unknown"]);
  });
});
