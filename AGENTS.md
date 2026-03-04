# krx-toolkit

## 목적 및 방향성

- DMA를 제외하고, 손 매매 + macro를 통해 관심종목을 추려내고 손으로 트레이딩을 하는 리테일의 관점에서 접근
- 한국 주식시장을 바라보는 관점을 `수급 기반 이벤트 매매`에 가깝다고 판단하여 완전 자동화보다 수동 주문을 돕는 도구로 방향 설정

## spec

- [ ] 매수 bet 사이즈 계산
- [ ] 주문 쪼개 넣기
- [ ] 진입 가격 대비 호가 잔량이 보유 수량보다 줄어들 경우의 본절 자동 트리거
- [ ] 변동성 기반 stop-loss 알람 (os-native way)

## todos

- [ ] egui 및 기타 필요한 deps 파악
- [ ] multiple viewport 지원을 위한 부모 viewport 생성
- [ ] 부모 viewport에 dark/light mode 지원
- [ ] 플래그에 따라 puppin 기반 통한 퍼포먼스 측정 도구 도입

## agent

- todos를 참고하여 구현을 시작할 것
- spec은 사용자를 위한 설명일 뿐이지 구현 목표가 아님
- ../egui/examples에 egui 관련 예시를 찾아볼 수 있음
- 차트, 호가, 체결 등 HTS 상에서 확인하는 것이 더 효율적인 것은 구현 금지
