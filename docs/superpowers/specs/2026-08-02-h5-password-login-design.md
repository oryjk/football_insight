# H5 Password Login Design

## Goal

Allow browser users to test authenticated H5 features without relying on WeChat OAuth. Login behavior is selected by the uni-app build platform:

- H5 uses account identifier and password login.
- WeChat Mini Program continues to use mini-program WeChat login.
- Both paths produce the existing access token and then share the same authenticated application flow.

This design covers ordinary authenticated features such as membership display, ticket monitoring, seat swap, team support, and AI chat. WeChat-only capabilities such as account binding, WeChat Pay, and subscription messages remain Mini Program test cases.

## Current State

The backend and frontend API layer already support password authentication through `POST /api/v1/auth/login`. The user page currently detects H5 but disables the guest login control and labels it `小程序登录`, so browser users cannot reach the existing password login API from the UI.

Users without a WeChat binding can still receive ordinary membership benefits. No fake WeChat identity or authentication bypass is required.

## User Experience

On the guest user page:

- An H5 build shows an enabled `账号登录` action.
- Selecting it opens a login sheet containing an account identifier input, password input, and submit button.
- A Mini Program build keeps the existing `去登录` action and mini-program WeChat login flow.
- While password login is in progress, the submit button is disabled and shows a loading state.
- On success, the sheet closes, the user state is updated, and any existing post-login redirect is consumed.
- On failure, the sheet stays open and shows the normalized backend error through the existing toast mechanism.

The authenticated account-management copy also reflects the platform: H5 tells users they can sign in again with their account and password, while the Mini Program continues to refer to WeChat login.

Registration and password reset are intentionally out of scope for this change. Test accounts are created through existing registration or administration capabilities, and credentials are never committed to frontend source or environment files.

## Component Boundaries

The change stays in the frontend user/account surface:

- `src/pages/user/index.vue` owns platform-specific entry selection, login-sheet state, submission, and post-login UI refresh.
- `src/api/auth.ts` remains the authentication API boundary; its existing `login` function is reused unchanged unless tests expose a contract gap.
- Existing access-token storage, current-user loading, and post-login redirect utilities remain the shared downstream flow.
- The Rust backend requires no new endpoint and no authentication behavior change.

Platform selection should use uni-app compile-time platform guards so Mini Program bundles do not expose H5 form behavior and H5 does not invoke `uni.login` for mini-program authentication.

## Data Flow

### H5

1. Guest selects `账号登录`.
2. The page validates that account identifier and password are present.
3. The frontend calls `POST /api/v1/auth/login` through the existing `login` API function.
4. The API function stores the returned access token.
5. The page updates the current user from the response and follows the existing post-login redirect.
6. Authenticated API calls continue to attach the same bearer token used by Mini Program sessions.

### WeChat Mini Program

1. Guest selects `去登录`.
2. The existing `uni.login` and mini-program WeChat login/bind flow runs unchanged.
3. The returned access token enters the same authenticated flow as H5.

## Validation And Errors

- Empty account identifier or password is rejected before the request.
- Password input uses masked entry and supports browser submission from the keyboard where uni-app permits it.
- Repeated submission is blocked while a request is pending.
- Backend authentication errors use `extractApiErrorMessage` and do not reveal whether an account exists.
- Closing the sheet clears the password. The account identifier may remain for retry within the current page session.
- Logout and expired-token behavior continue to clear the stored token and return to the platform-appropriate guest login entry.

## Testing

Focused frontend tests should cover:

- H5 resolves the guest action to password login and Mini Program resolves it to WeChat login.
- Empty form validation does not call the API.
- Successful password login updates the user, closes the sheet, and preserves post-login redirect behavior.
- Failed password login leaves the sheet open and releases the loading state.
- H5 build succeeds with Bun.
- Mini Program build succeeds with Bun, guarding against platform regressions.

Manual verification uses dedicated password accounts with representative membership tiers such as V1, V3, and V9. These are ordinary database users rather than hard-coded frontend personas. Real WeChat binding, payment, and subscription-message behavior is verified separately in the Mini Program.

## Security And Deployment

- No passwordless endpoint, JWT injection control, embedded credentials, or test-only backend bypass is added.
- H5 development continues to use the configured API base URL; production builds continue to use the production API base URL.
- This is a frontend-only change unless verification identifies an existing backend contract defect.
