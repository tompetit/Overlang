#include "ov_types.h"
#include <stdio.h>

uint8_t ov_test_t_db_params_cf14f30e(void);
uint8_t ov_test_t_db_parse_table_e06328e6(void);
uint8_t ov_test_t_db_roundtrip_e62d0424(void);
uint8_t ov_test_t_db_schema_68953109(void);
uint8_t ov_test_t_m_activity_17e4e90f(void);
uint8_t ov_test_t_m_issue_crud_3a06a7bd(void);
uint8_t ov_test_t_m_pure_helpers_e4f747a8(void);
uint8_t ov_test_t_m_search_bd3cac84(void);
uint8_t ov_test_t_m_validators_97a2de95(void);

int main(int argc, char** argv) {
  ov_init(argc, argv);
  int ovfz = ovfz_main(argc, argv);
  if (ovfz >= 0) return ovfz;
  ov_test_t_db_parse_table_e06328e6();
  ov_println(ovfz_json ? ov_str_new("{\"test\": \"t_db_parse_table\", \"status\": \"ok\"}", 44) : ov_str_new("ok t_db_parse_table", 19));
  ov_test_t_db_params_cf14f30e();
  ov_println(ovfz_json ? ov_str_new("{\"test\": \"t_db_params\", \"status\": \"ok\"}", 39) : ov_str_new("ok t_db_params", 14));
  ov_test_t_db_roundtrip_e62d0424();
  ov_println(ovfz_json ? ov_str_new("{\"test\": \"t_db_roundtrip\", \"status\": \"ok\"}", 42) : ov_str_new("ok t_db_roundtrip", 17));
  ov_test_t_db_schema_68953109();
  ov_println(ovfz_json ? ov_str_new("{\"test\": \"t_db_schema\", \"status\": \"ok\"}", 39) : ov_str_new("ok t_db_schema", 14));
  ov_test_t_m_validators_97a2de95();
  ov_println(ovfz_json ? ov_str_new("{\"test\": \"t_m_validators\", \"status\": \"ok\"}", 42) : ov_str_new("ok t_m_validators", 17));
  ov_test_t_m_pure_helpers_e4f747a8();
  ov_println(ovfz_json ? ov_str_new("{\"test\": \"t_m_pure_helpers\", \"status\": \"ok\"}", 44) : ov_str_new("ok t_m_pure_helpers", 19));
  ov_test_t_m_issue_crud_3a06a7bd();
  ov_println(ovfz_json ? ov_str_new("{\"test\": \"t_m_issue_crud\", \"status\": \"ok\"}", 42) : ov_str_new("ok t_m_issue_crud", 17));
  ov_test_t_m_search_bd3cac84();
  ov_println(ovfz_json ? ov_str_new("{\"test\": \"t_m_search\", \"status\": \"ok\"}", 38) : ov_str_new("ok t_m_search", 13));
  ov_test_t_m_activity_17e4e90f();
  ov_println(ovfz_json ? ov_str_new("{\"test\": \"t_m_activity\", \"status\": \"ok\"}", 40) : ov_str_new("ok t_m_activity", 15));
  return 0;
}
