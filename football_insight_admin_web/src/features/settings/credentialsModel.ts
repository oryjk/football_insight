import type { CredentialsPayload } from "../../api";
export interface CredentialsForm {
  username: string;
  currentPassword: string;
  newPassword: string;
  confirmPassword: string;
}
export class CredentialsFormError extends Error {
  constructor(
    public field: keyof CredentialsForm,
    message: string,
  ) {
    super(message);
  }
}
export function newCredentialsForm(username: string): CredentialsForm {
  return {
    username,
    currentPassword: "",
    newPassword: "",
    confirmPassword: "",
  };
}
export function credentialsPayload(
  form: CredentialsForm,
  currentUsername: string,
): CredentialsPayload {
  const username = form.username.trim().toLowerCase();
  if (!/^[a-z0-9._-]{3,64}$/.test(username))
    throw new CredentialsFormError(
      "username",
      "管理员账号须为 3–64 位英文字母、数字、点、下划线或连字符",
    );
  if (!form.currentPassword)
    throw new CredentialsFormError("currentPassword", "请输入当前密码");
  if (
    form.newPassword &&
    ([...form.newPassword].length < 8 || [...form.newPassword].length > 128)
  )
    throw new CredentialsFormError("newPassword", "新密码须为 8–128 个字符");
  if (form.newPassword && form.newPassword !== form.confirmPassword)
    throw new CredentialsFormError("confirmPassword", "确认密码与新密码不一致");
  if (
    username === currentUsername.trim().toLowerCase() &&
    (!form.newPassword || form.newPassword === form.currentPassword)
  )
    throw new CredentialsFormError(
      "username",
      "请修改管理员账号或设置不同的新密码",
    );
  return {
    username,
    current_password: form.currentPassword,
    ...(form.newPassword ? { new_password: form.newPassword } : {}),
  };
}

export function completeCredentialsChange(
  session: { invalidate: (token: string) => boolean },
  token: string,
  username: string,
): { username: string; message: string } | null {
  if (!session.invalidate(token)) return null;
  return { username, message: "管理员账号已更新，请使用新账号和密码登录" };
}
