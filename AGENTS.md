# krx-toolkit

## 목적 및 방향성

- DMA를 제외하고, 손 매매 + macro를 통해 관심종목을 추려내고 손으로 트레이딩을 하는 리테일의 관점에서 접근
- 한국 주식시장을 바라보는 관점을 `수급 기반 이벤트 매매`로 두고 완전 자동화보다 수동 주문을 돕는 도구로 방향 설정

## spec

- [ ] 진입 가격 대비 호가 잔량이 보유 수량보다 줄어들 경우의 본절 자동 트리거
- [ ] 매수 bet 사이즈 계산
- [ ] 주문 쪼개 넣기
- [ ] 변동성 기반 stop-loss 알람 (os-native way)

## todos

- [x] egui 및 기타 필요한 deps 파악
- [x] 한국어 지원 font 설정
- [x] zero margin, zero padding
- [x] always on top 버튼  
- [x] multiple viewport 지원을 위한 부모 viewport 생성
- [x] 자식 viewport 생성하기
- [x] viewport layer를 top으로 한번에 올리는 버튼
- [x] 종료시 confirm 묻기
- [x] reqwest client 모듈화
- [x] 키움 rest api 등록 및 토큰 refresh 자동화
- [x] 멀티창 기능
- [x] 키움 wss 연결
- [x] 키움 wss 연결 후 앱 레벨의 ping 대응
- [x] 키움 connectivity 확인 및 하단 상태바 연동
- [x] 마스터 파일 fetching
- [x] 종목 검색 모듈 (호가창을 여러개 띄울 수 있으므로 화면간 동기화는 지원하지 않음)
- [ ] 시세 모듈 (어느 한 화면에서 요청한다면, control panel에서 응집하여 구독하고 하위 파일로 카피하여 데이터를 흘려보내는 패턴)
- [ ] 로그 모듈
- [ ] 화면별 로그 파일 생성
- [ ] 부모 viewport에 dark/light mode 지원
- [ ] 자식 viewport 정렬 기능
- [ ] 플래그에 따라 puppin 기반 통한 퍼포먼스 측정 도구 도입
- [ ] 현재 계좌 fetching 및 예수금 

## agent

- virtual account는 사용하지 않는다. 주문 구분자를 넣을 수 없기에 구현 불가능하다.
- todos를 참고하여 구현을 시작할 것. spec은 사용자를 위한 설명일 뿐이지 구현 목표가 아님
- ../egui/examples에 egui 관련 예시를 찾아볼 수 있음
  - 현 프로젝트는 eframe = "0.33.3" 를 사용하고 있는데 예시는 이전 버전을 사용하고 있어 인터페이스가 다르니 주의.
- 차트, 호가, 체결 등 HTS 상에서 확인하는 것이 더 효율적인 것은 구현 금지


## 특이 로그


[src/api/kiwoom/ws.rs:113:29] &v = Object {
    "code": String("R00000"),
    "message": String("[장중 거래정지 지정/제개]261220_AL   |03"),
    "trnm": String("SYSTEM"),
}
[src/api/kiwoom/ws.rs:113:29] &v = Object {
    "code": String("R00000"),
    "message": String("[장중 거래정지 지정/제개]261220      |03"),
    "trnm": String("SYSTEM"),
}
