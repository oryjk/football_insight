import {
  useCallback,
  useEffect,
  useMemo,
  useRef,
  useState,
  type FormEvent,
} from "react";
import { LogIn, Moon, Sun, Trophy } from "lucide-react";
import { createApi, type Admin, type AdminSetupStatus } from "./api";
import { ApiClient, errorMessage, isAborted } from "./apiClient";
import { adminApiBaseForMount } from "./deploymentModel";
import { AuthSession, browserSessionStorage } from "./authSession";
import { applyTheme, readTheme } from "./theme";
import { Button } from "./components/ui/button";
import { Input } from "./components/ui/input";
import { Label } from "./components/ui/label";
import { Card, CardContent } from "./components/ui/card";
import { AdminShell, type AdminPage } from "./layout/AdminShell";
import { ProductsPage } from "./features/products/ProductsPage";
import { TopicReadersPage } from "./features/topics/TopicReadersPage";
import { ReviewExemptPage } from "./features/review/ReviewExemptPage";
import { BrandPage } from "./features/brand/BrandPage";
import { SupportBasesPage } from "./features/support/SupportBasesPage";
import { SettingsPage } from "./features/settings/SettingsPage";
import { StudentConfigPage } from "./features/student/StudentConfigPage";
import { completeCredentialsChange } from "./features/settings/credentialsModel";
import { AdminSetupForm } from "./features/setup/AdminSetupForm";
const session = new AuthSession(browserSessionStorage());
export function App() {
  const [token, setToken] = useState(session.getToken());
  const [setupStatus, setSetupStatus] = useState<AdminSetupStatus | null>(null);
  const [setupLoading, setSetupLoading] = useState(true);
  const [setupError, setSetupError] = useState("");
  const [setupRevision, setSetupRevision] = useState(0);
  const [admin, setAdmin] = useState<Admin | null>(null);
  const [verifying, setVerifying] = useState(!!token);
  const [authError, setAuthError] = useState("");
  const [authSuccess, setAuthSuccess] = useState("");
  const [page, setPage] = useState<AdminPage>("products");
  const [credentialsBusy, setCredentialsBusy] = useState(false);
  const credentialsLock = useRef(false);
  const [productBusy, setProductBusy] = useState(false);
  const productLock = useRef(false);
  const [theme, setTheme] = useState(readTheme);
  const [username, setUsername] = useState("");
  const [password, setPassword] = useState("");
  const [busy, setBusy] = useState(false);
  const request = useRef<AbortController | null>(null);
  const submitLock = useRef(false);
  const onCredentialsBusy = useCallback((value: boolean) => {
    credentialsLock.current = value;
    setCredentialsBusy(value);
  }, []);
  const onProductBusy = useCallback((value: boolean) => {
    productLock.current = value;
    setProductBusy(value);
  }, []);
  const api = useMemo(
    () =>
      createApi(
        new ApiClient(
          adminApiBaseForMount(import.meta.env.BASE_URL),
          (requestToken) => {
            if (session.invalidate(requestToken)) {
              setToken(null);
              setAdmin(null);
              setVerifying(false);
              setAuthError("登录已过期，请重新登录");
              setPassword("");
              setAuthSuccess("");
              credentialsLock.current = false;
              setCredentialsBusy(false);
              productLock.current = false;
              setProductBusy(false);
            }
          },
        ),
      ),
    [],
  );
  useEffect(() => {
    const controller = new AbortController();
    setSetupLoading(true);
    setSetupError("");
    api
      .setupStatus(controller.signal)
      .then((status) => {
        if (controller.signal.aborted) return;
        setSetupStatus(status);
        if (!status.needs_setup && session.getToken()) setVerifying(true);
        if (status.needs_setup) {
          const storedToken = session.getToken();
          if (storedToken) session.invalidate(storedToken);
          setToken(null);
          setAdmin(null);
          setVerifying(false);
          setPassword("");
        }
      })
      .catch((error) => {
        if (!controller.signal.aborted) {
          setSetupStatus(null);
          setSetupError(errorMessage(error));
          setVerifying(false);
        }
      })
      .finally(() => {
        if (!controller.signal.aborted) setSetupLoading(false);
      });
    return () => controller.abort();
  }, [api, setupRevision]);
  useEffect(() => {
    if (!token || !setupStatus || setupStatus.needs_setup) return;
    const controller = new AbortController();
    setVerifying(true);
    api
      .me(token, controller.signal)
      .then((value) => {
        if (!controller.signal.aborted && session.getToken() === token) {
          setAdmin(value);
          setAuthError("");
        }
      })
      .catch((error) => {
        if (!controller.signal.aborted && session.getToken() === token) {
          setAdmin(null);
          setAuthError(errorMessage(error));
        }
      })
      .finally(() => {
        if (!controller.signal.aborted) setVerifying(false);
      });
    return () => controller.abort();
  }, [api, token, setupStatus]);
  useEffect(() => () => request.current?.abort(), []);
  const toggleTheme = () =>
    setTheme((current) => {
      const next = current === "dark" ? "light" : "dark";
      applyTheme(next);
      return next;
    });
  async function login(event: FormEvent) {
    event.preventDefault();
    if (submitLock.current) return;
    submitLock.current = true;
    setBusy(true);
    setAuthError("");
    setAuthSuccess("");
    request.current?.abort();
    const controller = new AbortController();
    request.current = controller;
    try {
      const result = await api.login(
        username.trim(),
        password,
        controller.signal,
      );
      if (!controller.signal.aborted) {
        session.setToken(result.access_token);
        setPassword("");
        setPage("products");
        setToken(result.access_token);
      }
    } catch (error) {
      if (!isAborted(error)) setAuthError(errorMessage(error));
    } finally {
      submitLock.current = false;
      setBusy(false);
    }
  }
  async function logout() {
    if (
      submitLock.current ||
      credentialsLock.current ||
      productLock.current ||
      !token
    )
      return;
    submitLock.current = true;
    setBusy(true);
    setAuthError("");
    try {
      await api.logout(token);
      if (session.invalidate(token)) {
        setToken(null);
        setAdmin(null);
      }
    } catch (error) {
      if (session.getToken() === token) setAuthError(errorMessage(error));
    } finally {
      submitLock.current = false;
      setBusy(false);
    }
  }
  function credentialsUpdated(requestToken: string, updatedUsername: string) {
    const loginState = completeCredentialsChange(
      session,
      requestToken,
      updatedUsername,
    );
    if (!loginState) return;
    setToken(null);
    setAdmin(null);
    setVerifying(false);
    setUsername(loginState.username);
    setPassword("");
    setAuthError("");
    setAuthSuccess(loginState.message);
    setPage("products");
    onCredentialsBusy(false);
  }
  function retrySetup() {
    setSetupStatus(null);
    setSetupLoading(true);
    setSetupError("");
    setSetupRevision((value) => value + 1);
  }
  function setupCompleted() {
    const storedToken = session.getToken();
    if (storedToken) session.invalidate(storedToken);
    setToken(null);
    setAdmin(null);
    setVerifying(false);
    setUsername("superadmin");
    setPassword("");
    setAuthError("");
    setAuthSuccess("管理员已初始化，请使用 superadmin 和刚设置的密码登录");
    retrySetup();
  }
  function setupAlreadyInitialized() {
    setAuthSuccess("");
    setAuthError("");
    setUsername("superadmin");
    setPassword("");
    retrySetup();
  }
  const themeControl = (
    <Button
      variant="ghost"
      size="icon"
      aria-label={theme === "dark" ? "切换浅色主题" : "切换深色主题"}
      onClick={toggleTheme}
    >
      {theme === "dark" ? <Sun /> : <Moon />}
    </Button>
  );
  if (admin && token && setupStatus?.needs_setup === false)
    return (
      <AdminShell
        admin={admin}
        themeControl={themeControl}
        onLogout={logout}
        loggingOut={busy}
        page={page}
        navigationLocked={credentialsBusy || productBusy || busy}
        onNavigate={(next) => {
          if (
            !credentialsLock.current &&
            !productLock.current &&
            !submitLock.current
          )
            setPage(next);
        }}
      >
        {authError && (
          <p role="alert" className="operation-error">
            {authError}
          </p>
        )}
        {page === "products" ? (
          <ProductsPage
            key={token}
            api={api}
            token={token}
            onBusyChange={onProductBusy}
          />
        ) : page === "supportBases" ? (
          <SupportBasesPage
            key={token}
            api={api}
            token={token}
            onBusyChange={onProductBusy}
          />
        ) : page === "studentConfig" ? (
          <StudentConfigPage
            key={token}
            api={api}
            token={token}
            onBusyChange={onProductBusy}
          />
        ) : page === "brand" ? (
          <BrandPage
            key={token}
            api={api}
            token={token}
            onBusyChange={onProductBusy}
          />
        ) : page === "reviewExempt" ? (
          <ReviewExemptPage
            key={token}
            api={api}
            token={token}
            onBusyChange={onProductBusy}
          />
        ) : page === "topicReaders" ? (
          <TopicReadersPage key={token} api={api} token={token} />
        ) : (
          <SettingsPage
            key={token}
            admin={admin}
            api={api}
            token={token}
            onBusyChange={onCredentialsBusy}
            onUpdated={credentialsUpdated}
          />
        )}
      </AdminShell>
    );
  return (
    <main className="login-page">
      <div className="login-theme-control">{themeControl}</div>
      <Card className="login-card">
        <CardContent>
          <div className="login-brand">
            <Trophy aria-hidden="true" />
            <div>
              <h1>足球洞察</h1>
              <p>管理后台</p>
            </div>
          </div>
          <p className="page-description">使用管理员账户维护会员微信道具。</p>
          {setupLoading || verifying ? (
            <p role="status" className="empty-state">
              {setupLoading
                ? "正在检查管理员初始化状态…"
                : "正在验证管理员会话…"}
            </p>
          ) : setupError ? (
            <div className="login-form">
              <p className="operation-error" role="alert">
                {setupError}
              </p>
              <Button variant="secondary" onClick={retrySetup}>
                重新检查
              </Button>
            </div>
          ) : setupStatus?.needs_setup ? (
            <AdminSetupForm
              api={api}
              onSuccess={setupCompleted}
              onAlreadyInitialized={setupAlreadyInitialized}
            />
          ) : (
            <form onSubmit={login} className="login-form">
              <div className="form-field">
                <Label htmlFor="username">管理员账号</Label>
                <Input
                  id="username"
                  name="username"
                  autoComplete="username"
                  value={username}
                  onChange={(e) => setUsername(e.target.value)}
                  required
                  disabled={busy}
                />
              </div>
              <div className="form-field">
                <Label htmlFor="password">密码</Label>
                <Input
                  id="password"
                  name="password"
                  type="password"
                  autoComplete="current-password"
                  value={password}
                  onChange={(e) => setPassword(e.target.value)}
                  required
                  disabled={busy}
                />
              </div>
              {authSuccess && (
                <p role="status" className="operation-success">
                  {authSuccess}
                </p>
              )}
              {authError && (
                <p role="alert" className="operation-error">
                  {authError}
                </p>
              )}
              <Button size="lg" type="submit" disabled={busy}>
                <LogIn />
                {busy ? "正在登录…" : "登录"}
              </Button>
            </form>
          )}
        </CardContent>
      </Card>
    </main>
  );
}
