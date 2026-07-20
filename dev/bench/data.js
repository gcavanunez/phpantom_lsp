window.BENCHMARK_DATA = {
  "lastUpdate": 1784559473262,
  "repoUrl": "https://github.com/gcavanunez/phpantom_lsp",
  "entries": {
    "PHPantom Benchmarks": [
      {
        "commit": {
          "author": {
            "email": "anders@jenbo.dk",
            "name": "Anders Jenbo",
            "username": "AJenbo"
          },
          "committer": {
            "email": "anders@jenbo.dk",
            "name": "Anders Jenbo",
            "username": "AJenbo"
          },
          "distinct": true,
          "id": "10b9d2e062834abef4f790249bd5178e5aaab874",
          "message": "Add task for more Mago migration",
          "timestamp": "2026-07-20T08:24:36+02:00",
          "tree_id": "e037a2afe8ee007717a7ab3dced5669829665812",
          "url": "https://github.com/gcavanunez/phpantom_lsp/commit/10b9d2e062834abef4f790249bd5178e5aaab874"
        },
        "date": 1784559473010,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "cold_start_completion",
            "value": 6.258,
            "range": "± 0.196",
            "unit": "ms"
          },
          {
            "name": "completion_simple_class",
            "value": 0.043,
            "range": "± 0.004",
            "unit": "ms"
          },
          {
            "name": "completion_inheritance_depth/depth_5",
            "value": 0.113,
            "range": "± 0.007",
            "unit": "ms"
          },
          {
            "name": "completion_inheritance_depth/depth_10",
            "value": 0.166,
            "range": "± 0.007",
            "unit": "ms"
          },
          {
            "name": "completion_inheritance_depth/depth_20",
            "value": 0.25,
            "range": "± 0.011",
            "unit": "ms"
          },
          {
            "name": "completion_classmap_size/100_classes",
            "value": 0.258,
            "range": "± 0.011",
            "unit": "ms"
          },
          {
            "name": "completion_classmap_size/500_classes",
            "value": 0.975,
            "range": "± 0.026",
            "unit": "ms"
          },
          {
            "name": "completion_classmap_size/1000_classes",
            "value": 1.824,
            "range": "± 0.076",
            "unit": "ms"
          },
          {
            "name": "completion_generics_and_mixins",
            "value": 0.113,
            "range": "± 0.008",
            "unit": "ms"
          },
          {
            "name": "completion_with_narrowing",
            "value": 0.056,
            "range": "± 0.005",
            "unit": "ms"
          },
          {
            "name": "completion_5_method_chain",
            "value": 0.047,
            "range": "± 0.004",
            "unit": "ms"
          },
          {
            "name": "completion_cross_file_type_hint",
            "value": 0.064,
            "range": "± 0.005",
            "unit": "ms"
          },
          {
            "name": "completion_carbon_class",
            "value": 4.103,
            "range": "± 0.033",
            "unit": "ms"
          },
          {
            "name": "completion_yii_deep_hierarchy",
            "value": 0.208,
            "range": "± 0.003",
            "unit": "ms"
          },
          {
            "name": "completion_large_file",
            "value": 0.276,
            "range": "± 0.017",
            "unit": "ms"
          },
          {
            "name": "completion_short_file",
            "value": 0.076,
            "range": "± 0.008",
            "unit": "ms"
          },
          {
            "name": "variable_completion/short",
            "value": 0.051,
            "range": "± 0.004",
            "unit": "ms"
          },
          {
            "name": "variable_completion/long",
            "value": 0.123,
            "range": "± 0.012",
            "unit": "ms"
          },
          {
            "name": "hover_method_call",
            "value": 0.111,
            "range": "± 0.010",
            "unit": "ms"
          },
          {
            "name": "goto_definition_method",
            "value": 0.097,
            "range": "± 0.009",
            "unit": "ms"
          },
          {
            "name": "update_ast_parse_time/100_lines",
            "value": 0.199,
            "range": "± 0.001",
            "unit": "ms"
          },
          {
            "name": "update_ast_parse_time/500_lines",
            "value": 1.076,
            "range": "± 0.015",
            "unit": "ms"
          },
          {
            "name": "update_ast_parse_time/2000_lines",
            "value": 5.652,
            "range": "± 0.143",
            "unit": "ms"
          },
          {
            "name": "reparse_500_line_file",
            "value": 1.093,
            "range": "± 0.023",
            "unit": "ms"
          },
          {
            "name": "diagnostics/fixture/lots_of_new_generic_objects",
            "value": 0.039,
            "range": "± 0.001",
            "unit": "ms"
          },
          {
            "name": "diagnostics/fixture/lots_of_new_objects",
            "value": 0.036,
            "range": "± 0.002",
            "unit": "ms"
          },
          {
            "name": "diagnostics/fixture/lots_of_missing_methods",
            "value": 73.278,
            "range": "± 0.240",
            "unit": "ms"
          },
          {
            "name": "diagnostics/fixture/method_chain",
            "value": 1.701,
            "range": "± 0.023",
            "unit": "ms"
          }
        ]
      }
    ]
  }
}