# UPnP NAT Traversal 테스트

libp2p UPnP를 사용하여 NAT 뒤에서 자동 포트 매핑과 P2P 연결을 테스트하는 프로그램입니다.

## 목표

1. UPnP로 라우터에 자동 포트 매핑 요청
2. 두 개의 NAT 뒤 디바이스 간 직접 연결 테스트
3. 포트 매핑 성공 여부 확인

## 빌드

```bash
cd upnp-test
cargo build --release
```

## 사용 방법

### 1단계: 디바이스 A 실행 (NAT 뒤)

첫 번째 디바이스에서 프로그램을 실행합니다:

```bash
cargo run -- --port 4001
```

또는 릴리스 빌드로:

```bash
cargo run --release -- --port 4001
```

출력 예시:
```
🚀 Starting UPnP NAT Traversal Test
📝 Listening port: 4001
🆔 Local PeerID: 12D3KooWAbcd1234...
👂 Listening on: /ip4/0.0.0.0/tcp/4001
📍 New listen address: /ip4/192.168.1.100/tcp/4001
🎉 UPnP: Successfully mapped external address: /ip4/203.0.113.50/tcp/4001
✅ This address can be used by other peers to connect to you!
```

**중요**: UPnP 매핑 성공 시 출력된 외부 주소를 복사합니다:
- 예: `/ip4/203.0.113.50/tcp/4001`
- 이 주소에 PeerID를 추가: `/ip4/203.0.113.50/tcp/4001/p2p/12D3KooWAbcd1234...`

### 2단계: 디바이스 B 실행 (NAT 뒤)

두 번째 디바이스(또는 다른 터미널)에서 디바이스 A에 연결합니다:

```bash
cargo run -- --port 4002 --connect /ip4/203.0.113.50/tcp/4001/p2p/12D3KooWAbcd1234...
```

출력 예시:
```
🚀 Starting UPnP NAT Traversal Test
📝 Listening port: 4002
🆔 Local PeerID: 12D3KooWXyz7890...
👂 Listening on: /ip4/0.0.0.0/tcp/4002
🔗 Attempting to connect to: /ip4/203.0.113.50/tcp/4001/p2p/12D3KooWAbcd1234...
✅ Dial initiated successfully
🎉 UPnP: Successfully mapped external address: /ip4/198.51.100.75/tcp/4002
📥 Incoming connection from ...
🤝 Connection established with peer: 12D3KooWAbcd1234... at /ip4/203.0.113.50/tcp/4001
🔍 Identified peer: 12D3KooWAbcd1234...
🏓 Ping to 12D3KooWAbcd1234... succeeded: RTT = 45ms
```

## 검증 포인트

### ✅ UPnP 성공 시나리오

1. **포트 매핑 성공**
   ```
   🎉 UPnP: Successfully mapped external address: /ip4/<공인IP>/tcp/<포트>
   ```

2. **연결 성공**
   ```
   🤝 Connection established with peer: <PeerID>
   ```

3. **피어 식별**
   ```
   🔍 Identified peer: <PeerID>
   ```

4. **Ping 성공**
   ```
   🏓 Ping to <PeerID> succeeded: RTT = <시간>
   ```

### ⚠️ UPnP 실패 시나리오

1. **UPnP 게이트웨이 없음**
   ```
   ⚠️  UPnP: No UPnP gateway found on network
       - Make sure your router supports UPnP/IGD
       - Check if UPnP is enabled in router settings
   ```
   
   **해결 방법**: 라우터 설정에서 UPnP/IGD 활성화

2. **라우팅 불가 게이트웨이**
   ```
   ⚠️  UPnP: Gateway is not routable
       - Your router may be behind another NAT (carrier-grade NAT)
   ```
   
   **해결 방법**: ISP에 공인 IP 요청 또는 relay 사용

## 명령줄 옵션

```bash
upnp-test [OPTIONS]

Options:
  -p, --port <PORT>           리스닝 포트 [기본값: 4001]
  -c, --connect <CONNECT>     연결할 피어의 multiaddr (선택)
  -h, --help                  도움말 출력
```

## 로그 레벨 조정

환경 변수로 로그 레벨을 조정할 수 있습니다:

```bash
# 전체 상세 로그
RUST_LOG=debug cargo run -- --port 4001

# UPnP 모듈만 상세 로그
RUST_LOG=libp2p_upnp=debug cargo run -- --port 4001

# 정보만 출력 (기본값)
RUST_LOG=info cargo run -- --port 4001
```

## 테스트 체크리스트

- [ ] 디바이스 A에서 UPnP 매핑 성공 확인
- [ ] 디바이스 B에서 UPnP 매핑 성공 확인
- [ ] 두 디바이스가 서로 연결됨
- [ ] Identify 프로토콜로 피어 정보 교환
- [ ] Ping이 정상적으로 왕복

## 문제 해결

### 연결은 되지만 UPnP 매핑이 실패하는 경우

UPnP가 실패해도 libp2p는 다른 주소(로컬 IP)로 연결을 시도합니다. 같은 네트워크에 있다면 로컬 IP로 연결될 수 있습니다.

### 방화벽 문제

일부 방화벽이 연결을 차단할 수 있습니다:
- Windows: Windows Defender 방화벽에서 포트 허용
- Linux: `ufw allow <포트>` 또는 `iptables` 규칙 추가
- macOS: 시스템 환경설정 > 보안 및 개인 정보 보호 > 방화벽

### 로컬 네트워크 테스트

같은 네트워크에서 테스트할 때는 UPnP가 필요하지 않습니다. 로컬 IP 주소를 사용하세요:

```bash
# 디바이스 A (로컬 IP 확인)
cargo run -- --port 4001

# 디바이스 B (로컬 IP로 연결)
cargo run -- --port 4002 --connect /ip4/192.168.1.100/tcp/4001/p2p/<PeerID>
```

## 다음 단계

이 테스트가 성공하면, 메인 Chiral Network 프로젝트에 UPnP를 통합할 수 있습니다.

## 라이센스

MIT

