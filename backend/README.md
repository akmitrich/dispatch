# `Backend`

Предоставляет `REST API` и `WebSocket API` для авторизации пользователей и обмена сообщениями. Включает эндпоинты для регистрации, входа в систему и подключения через `WebSocket`, а также использует базу данных `PostgreSQL` для хранения данных.

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
    - **Тело:** `JWT` (JSON Web Token)
  - `409 Conflict`
    - **Описание:** Неверный формат запроса
    - **Тело:** `Wrong format`
  - `401 Unauthorized`
    - **Описание:** Неверное имя пользователя или пароль
    - **Тело:** `Wrong username/password`
  - `409 Conflict`
    - **Описание:** Пользователь уже подключен
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

**Первое сообщение:** `JWT` (JSON Web Token)

**`Responses`**  
- **Подключение установлено**    
- **Подключение закрыто**  
  - **Описание:** Неверный или истекший `JWT` (JSON Web Token)

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

## **`Setup`**

### **`Environments`**
В файле `.env` нужно указать свои значения вместо `default` для следующих переменных:
```bash
POSTGRES_USER
POSTGRES_PASSWORD
POSTGRES_DB
```

### **`Build and Run`**
Используйте `Docker Compose` для сборки и запуска приложения:

```bash
docker-compose up --build -d
```
