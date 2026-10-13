export interface SetupForm {
  password: string;
  confirmPassword: string;
}
export class SetupFormError extends Error {
  constructor(
    public field: keyof SetupForm,
    message: string,
  ) {
    super(message);
  }
}
export function newSetupForm(): SetupForm {
  return { password: "", confirmPassword: "" };
}
export function setupPayload(form: SetupForm): { password: string } {
  const length = [...form.password].length;
  if (length < 8 || length > 128)
    throw new SetupFormError("password", "密码须为 8–128 个字符");
  if (form.password !== form.confirmPassword)
    throw new SetupFormError("confirmPassword", "确认密码与设置的密码不一致");
  return { password: form.password };
}
