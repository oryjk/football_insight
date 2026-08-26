export class ApiRequestError extends Error {
  constructor(message: string, public statusCode = 0) {
    super(message)
  }
}
