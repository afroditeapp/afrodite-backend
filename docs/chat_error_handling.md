
The objective:

Detect was message really received by server in case HTTP
response will not be received by client.

The detection could be integrated into sending API and to separate
API (for resending to prevent sending the same message as a new message)

Successful case
1. client sends message with ID 0
2. server expects ID 0
3. client receives response and increments ID to 1
4. client sends message with ID 1
5. server expects ID 1
3. client receives response and increments ID to 2
and so on

Failure (successfull HTTP response lost)
1. client sends message with ID 0
2. server expects ID 0
3. response to client is lost - client keeps the same ID
4. client sends messsage with ID 0
5. server expects ID 1 (previous message was successful from server point of view)
6. client receives error response from server (includes the expected ID 1)
7. client marks the previous failed message as correctly sent

Failure (failed HTTP response lost)
1. client sends message with ID 0
2. server expects ID 0
3. failure response to client is lost - client keeps the same ID
4. client sends messsage with ID 0
5. server expects ID 0
6. client receives successfull response
7. client marks the previous failed message as really failed

Failure (successfull HTTP response lost and resend the same message)
1. client sends message with ID 0
2. server expects ID 0
3. response to client is lost - client keeps the same ID
4. user selects failed message resend
5. client asks the current expected ID from server
6. client receives ID 1 from server (the server actually received the message)
7. the message is marked as sent

Relogin of client
1. the successful case runs
2. logout and login - the ID resets back to 0
3. client sends message with ID 0
4. server expects ID 2 (from previous login)
5. client receives error response from server (includes the expected ID 2)
6. client detects that previous sent message is not failed
7. client resets the server side ID to 0

The ID is `SenderMessageId` type in the backend code.
