# `Backend`

## `Overview`
Предоставляет `REST API` и `WebSocket API` для авторизации пользователей и обмена сообщениями. Она включает эндпоинты для входа, регистрации и подключения через WebSocket, а также использует базу данных `PostgreSQL` для хранения данных.

## **`Endpoints`**
### **`POST /signin`**
`Content-Type: application/json`
```json
{
    "username": "default",
    "password": "default"
}
  ```
**`Responses`**
  - `200 OK `
    - **Описание:** Авторизация прошла успешно
    - **Тело:** `JWT`
  - `409 Conflict`
    - **Описание:** Неверный формат запроса
    - **Тело:** `Wrong format`
  - `401 Unauthorized`
    - **Описание:** Неверное имя пользователся или пароль
    - **Тело:** `Wrong username/password`
  - `409 Conflict`
    - **Описание:** Уже подключен
    - **Тело:** `Already in`

---

### **`POST /signup`**  
`Content-Type: application/json`  
```json
{
    "username": "default",
    "password": "default"
}
```  
**`Responses`**  
- `200 OK`  
  - **Описание:** Регистрация прошла успешно
  - **Тело:** `Success`
- `409 Conflict`  
  - **Описание:** Неверный формат запроса  
  - **Тело:** `Wrong format`  
- `409 Conflict`  
  - **Описание:** Пользователь уже зарегистрирован  
  - **Тело:** `Already exists`

---

### **`GET /connect`**  
`Upgrade: websocket`  

**Первое сообщение:** `JWT`.  

**`Responses`**  
- **Подключение установлено**    
- **Подключение закрыто:**  
  - **Описание** Неверный или истекший `JWT`.  

---

## **`Database Schema`**
### **`Users Table`**
```sql
CREATE TABLE IF NOT EXISTS users (
    username VARCHAR(8) PRIMARY KEY NOT NULL,
    password VARCHAR(64) NOT NULL,
    created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP
);
```
### **`Messages Table`**
```sql
CREATE TABLE IF NOT EXISTS messages (
    id SERIAL PRIMARY KEY,
    "from" VARCHAR(8) NOT NULL,
    body TEXT NOT NULL,
    timestamp BIGINT NOT NULL
);
```
---

## **`Setup`**

### **`Environment Configuration`**
В файле .env необходимо указать следующие значения:
```bash
POSTGRES_USER=default
POSTGRES_PASSWORD=default
POSTGRES_DB=default
```

### **`Run the Application`**
Используйте Docker Compose для сборки и запуска приложения:

```bash
docker-compose up --build -d
```