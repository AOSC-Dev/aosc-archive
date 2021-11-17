import os
import sys
import shutil
import file_
#使用方法：python3 file1.py 软件仓库路径 退休文件夹路径 数据库文件路径
#例如软件仓库路径为：pool/stable/main 退休文件夹路径为：archive/main 数据库路径：deb_information.db
#则使用方法则为：python3 file1.py pool/stable/main archive/main deb_information.db
#可能遇到的问题：1、archive/main下已创建空文件夹，需删去空文件夹；2、deb包命名不规范，字符串分割后无法刚好填入数据库中，导致退休中途停止。
def main(args):
    if os.path.exists(args[0]) and os.path.exists(args[1]) and os.path.exists(args[2]): 
        old_path=args[0]  #软件仓库路径
        new_path=args[1]   #退休文件的路径
        db=args[2]        #存放deb的数据库路径
        file_dict={}
        for i in os.listdir(old_path):
            os.makedirs(new_path+'/'+i)
        file_list=[]
        file_.get_oldpath(file_list)
        file_.abspath(file_list,file_dict)
        #for key in file_dict:
        #    print(key+':'+file_dict[key])
        empty_file=[]
        for i in file_list:
            if os.path.exists(i):
                if file_.SELECT(i,db):
                    shutil.move(i,new_path+'/'+file_dict[i]+'/'+i.split(sep='/')[-1])
                    print(i.split(sep='/')[-1]+'  has been moved to '+new_path+'/'+file_dict[i])
                else:
                    print(i.split(sep='/')[-1]+' has been retired')
            else:
                empty_file.append(i)
        #file_.rely_(new_path)
        print('file_list:')
        for i in empty_file:
            print(i)
    #else:
    #    print('Usage:'+sys.argv[0]+' stable_main_path'+' archive_main_path'+)

if __name__=='__main__':
    main(sys.argv[1:])
