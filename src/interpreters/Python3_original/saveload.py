# function has a complicated name so it NEVER gets overriden by user's defs
def sniprun142859_save(sniprun_filename='memo', globals_=None):

    globals_ = globals_ or globals()
    import klepto
    sniprun_dict_to_save = {}
    for key, value in globals_.items():
        if not key.startswith('__'):
            sniprun_dict_to_save[key] = value
            # print('added %s to save' % key)
    sniprun_a = klepto.archives.dir_archive(name=sniprun_filename, dict=sniprun_dict_to_save)
    sniprun_a.dump()


def sniprun142859_load(sniprun_filename='memo', globals_=None):
    import klepto
    sniprun_b = klepto.archives.dir_archive(sniprun_filename)
    for key in sniprun_b.archive.keys():
        sniprun_b.load(key)
        # print("loading %s " % key)
        globals()[key] = sniprun_b[key]
