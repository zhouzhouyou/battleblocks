"""Check the wizard ordinary form along with the protected original roster."""
from check_legacy_normal_unchanged import check, mutation_checks
import json
if __name__=='__main__':
    result=check();result['rejected_mutations']=mutation_checks();print(json.dumps(result,indent=2))
