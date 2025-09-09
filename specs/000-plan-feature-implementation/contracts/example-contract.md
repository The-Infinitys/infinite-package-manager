# Example Contract

This is a placeholder for an API contract. It should define the input and output for a specific API endpoint or interaction.

## Endpoint: /plan/create

- **Method**: POST
- **Request Body**:
  ```json
  {
    "feature_spec_path": "string"
  }
  ```
- **Response Body (Success 200)**:
  ```json
  {
    "plan_id": "string",
    "status": "success"
  }
  ```
- **Response Body (Error 400)**:
  ```json
  {
    "error": "string"
  }
  ```
